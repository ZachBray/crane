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
extern crate termion;
extern crate tui;

mod args;
mod timer;
mod hub;
mod local;
mod s3;
mod ui;

use args::parse_args;
use hub::GitHubClient;
use failure::Error;
use crate::hub::RepoLocator;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use crate::timer::RandomExpBackoffTimer;
use crate::hub::requests::SetStatusRequest;
use crate::hub::common::State;
use std::process::Command;
use crate::local::LocalRepo;
use crate::s3::Bucket;
use crate::ui::Property;
use crate::ui::Summary;
use std::thread;
use std::io;
use termion::input::TermRead;
use termion::event::Key;
use std::time::Duration;

const TICK_PERIOD: Duration = Duration::from_millis(64);

fn main() -> Result<(), Error> {
    let args = parse_args();

    let properties = vec![
        Property::new("Owner", &args.owner),
        Property::new("Repo", &args.repository),
        Property::new("Branch", &args.branch),
        Property::new("Build Type", &args.context),
    ];

    let mut ui = Summary::new(properties)?;

    let repo = RepoLocator {
        owner: args.owner,
        repo: args.repository,
    };


    let github = GitHubClient::new(&args.token)?;
    let mut local = LocalRepo::new(&args.user, &args.token, &repo, &args.branch, &args.context)?;
    let mut timer = RandomExpBackoffTimer::new();
    let bucket_key_prefix = format!("build/logs/{}/{}", &args.branch, &args.context);
    let bucket = Bucket::new(args.region, args.bucket, bucket_key_prefix);
    let is_running = monitor_application_state();
    while is_running() {
        if timer.is_due() {
            test_latest_commit(&github, &mut local, &repo,
                               &bucket, &mut ui, &args.context,
                               &args.script).unwrap_or_else(|e| {
                ui.record_error(e);
            });
            let due_time = timer.reset();
            ui.reset_retry_window(due_time);
        }

        ui.render()?;
        thread::sleep(TICK_PERIOD);
    }

    Ok(())
}

fn monitor_application_state() -> impl Fn() -> bool {
    let running = Arc::new(AtomicBool::new(true));
    let sig_int_running = running.clone();
    ctrlc::set_handler(move || {
        sig_int_running.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");
    let ctrl_c_running = running.clone();
    thread::spawn(move || {
       let input = io::stdin();
       for event in input.keys() {
           if let Ok(key) = event {
               match key {
                   Key::Ctrl('c') | Key::Char('q') =>
                       ctrl_c_running.store(false, Ordering::SeqCst),
                   _ => {}
               }
           }
       }
    });
    return move || running.load(Ordering::SeqCst);
}

fn test_latest_commit(github: &GitHubClient, local: &mut LocalRepo, repo: &RepoLocator,
                      bucket: &Bucket, ui: &mut Summary, context: &str, script: &str) -> Result<(), Error> {
    let maybe_commit = github.get_last_commit(&repo)?;
    if let Some(commit) = maybe_commit {
        let statuses = github.get_statuses(&commit)?;
        let maybe_status = statuses.iter()
            .filter(|existing_status|
                existing_status.context.as_ref().map_or(false, |c| c == context))
            .next();
        if let Some(status) = maybe_status {
            let ui_status = match status.state {
                State::Pending => ui::Status::Pending,
                State::Error | State::Failure => ui::Status::Failed,
                State::Success => ui::Status::Succeeded,
            };
            ui.record_build(&commit.sha, ui_status)
        } else {
            ui.record_build(&commit.sha, ui::Status::Pending);
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
                    ui.record_build(&commit.sha, ui::Status::Succeeded);
                    State::Success
                } else {
                    ui.record_build(&commit.sha, ui::Status::Failed);
                    State::Failure
                };
            bucket.put(&format!("{}/stdout.txt", commit.sha), process_output.stdout)?;
            bucket.put(&format!("{}/stderr.txt", commit.sha), process_output.stderr)?;
            let build_url = bucket.get_url(&commit.sha);
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
