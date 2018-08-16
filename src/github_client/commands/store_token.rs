use actix_web::actix::*;
use model::values::installation_id::InstallationId;
use github_client::state::GitHubClientState;
use github_client::state::AccessToken;

pub struct StoreToken {
    pub installation_id: InstallationId,
    pub access_token: AccessToken,
}

impl Message for StoreToken {
    type Result = ();
}

impl Handler<StoreToken> for GitHubClientState {
    type Result = ();

    fn handle(&mut self, msg: StoreToken, _ctx: &mut Context<Self>) -> () {
        self.access_tokens.insert(msg.installation_id, msg.access_token);
    }
}