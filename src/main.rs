extern crate clap;
extern crate ctrlc;
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate git2;
extern crate rand;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod args;
mod sleeper;
mod hub;

use args::parse_args;
use hub::GitHubClient;
use failure::Error;
use crate::hub::RepoLocator;
use crate::hub::requests::GetCommitsRequest;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use crate::sleeper::RandomSleeper;
use crate::hub::requests::SetStatusRequest;
use crate::hub::common::State;
use std::process::Command;
use std::fs;
use crate::local::LocalRepo;

fn main() -> Result<(), Error> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");
    println!("Press Ctrl-C to stop...");

    let args = parse_args();
    let github = GitHubClient::new(&args.token)?;
    let repo = RepoLocator {
        owner: args.owner,
        repo: args.repository,
    };

    let mut local = LocalRepo::new(&args.user, &args.token, &repo, &args.context)?;
    let mut sleeper = RandomSleeper::new();
    while running.load(Ordering::SeqCst) {
        test_latest_commit(&github, &mut local, &repo,
                           &args.branch,
                           &args.context,
                           &args.script).unwrap_or_else(|e| {
            println!("Failed to test latest commit. {}", e)
        });
        sleeper.sleep();
    }

    println!("Exiting...");
    Ok(())
}

fn test_latest_commit(github: &GitHubClient, local: &mut LocalRepo, repo: &RepoLocator,
                      branch: &str, context: &str, script: &str) -> Result<(), Error> {
    let branch_filter = GetCommitsRequest { sha: &branch };
    let maybe_commit = github.get_last_commit(&repo, branch_filter)?;
    if let Some(commit) = maybe_commit {
        println!("Last commit was: {}", commit.sha);
        let statuses = github.get_statuses(&commit)?;
        let has_status_already = statuses.iter().any(|existing_status|
            existing_status.context.as_ref().map_or(false, |c| c == context));
        if !has_status_already {
            println!("Starting test...");
            github.set_status(&commit, SetStatusRequest {
                state: State::Pending,
                target_url: None,
                description: Some("Running ..."), // TODO incorporate machine label
                context: Some(context),
            });
            local.reset_to(&commit)?;
            let path_to_script = format!("{}/{}", &local.path(), &script);
            let process_output = Command::new("bash")
                .arg(path_to_script)
                .output()?;
            // TODO write stdout/stderr to S3
            let new_state =
                if process_output.status.success() {
                    State::Failure
                } else {
                    State::Success
                };
            github.set_status(&commit, SetStatusRequest {
                state: new_state,
                target_url: None,
                description: None,
                context: Some(context),
            });
        }
        println!("TODO hasStatus: {:?}", has_status_already);
    }
    Ok(())
}

mod local {
    use crate::hub::RepoLocator;
    use failure::Error;
    use std::fs;
    use crate::hub::CommitLocator;
    use git2::Object;
    use git2::Oid;
    use git2::Repository;
    use git2::ResetType;

    pub struct LocalRepo {
        path: String,
        git: Repository,
    }

    impl LocalRepo {
        pub fn new(user: &str, token: &str, locator: &RepoLocator, context: &str) -> Result<Self, Error> {
            let url = format!("https://{}:{}@github.com/{}/{}.git", &user, &token, &locator.owner, &locator.repo);
            let path = format!("/tmp/crane/{}/{}/{}", &locator.owner, &locator.repo, &context);
            fs::remove_dir_all(&path);
            let repo = LocalRepo {
                path: path.clone(),
                git: Repository::clone(&url, &path)?,
            };
            Ok(repo)
        }

        pub fn reset_to(&mut self, commit: &CommitLocator) -> Result<(), Error> {
            self.git.find_remote("origin")?
                .fetch(&["refs/heads/*"], None, None)?;
            let git_commit = self.git.find_commit(Oid::from_str(&commit.sha)?)?;
            self.git.reset(&git_commit.as_object(), ResetType::Hard, None)?;
            Ok(())
        }

        pub fn path(&self) -> &str {
            &self.path
        }
    }
}
