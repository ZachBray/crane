extern crate actix_web;
extern crate core;
extern crate chrono;
extern crate clap;
extern crate env_logger;
extern crate fnv;
extern crate futures;
#[macro_use]
extern crate log;
#[macro_use]
extern crate quick_error;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate jsonwebtoken;

mod error;
mod model;
mod server;
mod hooks_api;
mod github_client;

use actix_web::actix::*;
use model::state::ApplicationState;
use server::start_web_server;
use clap::App;
use clap::Arg;
use github_client::state::GitHubClientState;

#[derive(Debug)]
struct Args {
    private_key_file: String,
}

const PRIVATE_KEY_ARG: &'static str = "private-key";

fn parse_args() -> Result<Args, error::Error> {
    let matches = App::new("craneci")
        .about("Continuous integration server")
        .arg(Arg::with_name(PRIVATE_KEY_ARG)
            .short("k")
            .value_name("DER_FILE")
            .takes_value(true)
            .help("Sets the GitHub App's private key")
            .required(true))
        .get_matches();
    let private_key_file = matches.value_of(PRIVATE_KEY_ARG)
        .unwrap()
        .to_string();
    Ok(Args {
        private_key_file
    })
}

fn configure_logger() {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();
}

fn main() -> Result<(), error::Error> {
    let args = parse_args()?;
    configure_logger();
    info!("Starting with: {:?}", args);
    let actor_system = System::new("crane");
    let github_client_state = GitHubClientState::new(args.private_key_file)?;
    let github_client_address = Arbiter::start(move |_|
        github_client_state);
    let app_state = ApplicationState::new(github_client_address);
    let app_actor_address = Arbiter::start(move |_|
        app_state);
    start_web_server(app_actor_address)?;
    actor_system.run();
    Ok(())
}
