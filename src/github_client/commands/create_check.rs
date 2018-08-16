use std::str;
use actix_web::actix::*;
use actix_web::client;
use actix_web::HttpMessage;
use actix_web::http::header;
use futures::Future;
use futures::future::ok;
use serde_json;
use model::values::installation_id::InstallationId;
use model::values::sha::Sha;
use error;
use github_client::state::GitHubClientState;

#[derive(Serialize)]
pub struct Output {
    pub title: &'static str,
    pub summary: &'static str,
}

#[derive(Serialize)]
pub struct Check {
    pub name: &'static str,
    pub head_sha: String,
    pub output: Output,
}

#[derive(Deserialize)]
pub struct CheckCreated {
    pub id: u32
}

pub struct CreateCheck<R>
    where R: FnOnce(Result<CheckCreated, error::Error>) -> () + 'static
{
    pub installation_id: InstallationId,
    pub repo_url: String,
    pub check: Check,
    pub reply_channel: R,
}


impl<R> Message for CreateCheck<R>
    where R: FnOnce(Result<CheckCreated, error::Error>) -> () + 'static
{
    type Result = ();
}

impl<R> Handler<CreateCheck<R>> for GitHubClientState
    where R: FnOnce(Result<CheckCreated, error::Error>) -> () + 'static
{
    type Result = ();

    fn handle(&mut self, msg: CreateCheck<R>, ctx: &mut Context<GitHubClientState>) -> () {
        let install_token =
            self.installation_token(&ctx.address(), msg.installation_id);
        let repo_url = msg.repo_url;
        let check = msg.check;
        let check_created =
            install_token.and_then(move |token| {
                let url = format!("{}/check-runs", repo_url);
                info!("Building create check run request: {}", url);
                let auth_header = format!("token {}", token);
                let payload = serde_json::to_vec(&check).unwrap(); // TODO
                client::post(url)
                    .header(header::ACCEPT, "application/vnd.github.antiope-preview+json")
                    .header(header::AUTHORIZATION, auth_header)
                    .body(payload)
                    .unwrap()
                    .send()
                    .from_err::<error::Error>()
                    .inspect(|response| info!("Received check run response with status: {}", response.status()))
                    .and_then(|response|
                        response.body()
                            .limit(100 * 1024)
                            .from_err::<error::Error>()
                            .map(|body| {
                                let body_str = str::from_utf8(body.as_ref()).unwrap_or("");
                                info!("Received: {}", body_str);
                                serde_json::from_slice::<CheckCreated>(body.as_ref())
                                    .map_err(|e| error::Error::ParseError(e))
                            }))
            });

        let reply_channel = msg.reply_channel;
        ctx.spawn(check_created
            .or_else(|e| ok(Err(e)))
            .map(move |result| reply_channel(result))
            .actfuture());
    }
}

