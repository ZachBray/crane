extern crate clap;
extern crate ctrlc;
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate git2;
extern crate rand;
extern crate reqwest;
extern crate rusoto_core;
extern crate rusoto_s3;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod args;
mod sleeper;
mod hub;
mod local;
mod s3;

use args::parse_args;
use hub::GitHubClient;
use failure::Error;
use crate::hub::RepoLocator;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use crate::sleeper::RandomSleeper;
use crate::hub::requests::SetStatusRequest;
use crate::hub::common::State;
use std::process::Command;
use crate::local::LocalRepo;
use crate::s3::Bucket;

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

    let mut local = LocalRepo::new(&args.user, &args.token, &repo, &args.branch, &args.context)?;
    let mut sleeper = RandomSleeper::new();
    let bucket_key_prefix = format!("build/logs/{}/{}", &args.branch, &args.context);
    let bucket = Bucket::new(args.region, args.bucket, bucket_key_prefix);
    while running.load(Ordering::SeqCst) {
        test_latest_commit(&github, &mut local, &repo,
                           &bucket,
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
                      bucket: &Bucket, context: &str, script: &str) -> Result<(), Error> {
    let maybe_commit = github.get_last_commit(&repo)?;
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
                description: None, // TODO incorporate machine label
                context: Some(context),
            })?;
            local.reset_to(&commit)?;
            let path_to_script = format!("{}/{}", &local.path(), &script);
            let process_output = Command::new("bash")
                .arg(path_to_script)
                .output()?;
            let new_state =
                if process_output.status.success() {
                    println!("Test successful! :-D");
                    State::Success
                } else {
                    println!("Test failed! :-(");
                    State::Failure
                };
            let build_url = bucket.put(&format!("{}/stdout.txt", commit.sha), process_output.stdout)?;
            bucket.put(&format!("{}/stderr.txt", commit.sha), process_output.stderr)?;
            github.set_status(&commit, SetStatusRequest {
                state: new_state,
                target_url: Some(&build_url),
                description: None,
                context: Some(context),
            })?;
        }
    }
    Ok(())
}
