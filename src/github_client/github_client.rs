use actix_web::actix::*;
use github_client::state::GitHubClientState;

impl Actor for GitHubClientState {
    type Context = Context<Self>;
}

pub type GitHubClient = Addr<GitHubClientState>;
