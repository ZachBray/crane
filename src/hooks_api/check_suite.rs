use actix_web::*;
use hooks_api::common::Installation;
use hooks_api::common::Repository;
use model::values::sha::Sha;
use model::commands::with_installation::WithInstallation;
use model::values::installation_id::InstallationId;
use model::commands::with_commit::WithCommit;
use model::commands::start_build::StartBuild;
use server::WebState;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum CheckSuiteAction {
    Requested,
    Rerequested,
    Completed
}

#[derive(Deserialize, Debug)]
pub struct CheckSuite {
    head_sha: Sha
}

#[derive(Deserialize, Debug)]
pub struct CheckSuiteEvent {
    installation: Installation,
    action: CheckSuiteAction,
    check_suite: CheckSuite,
    repository: Repository,
}

pub fn handle_check_suite((payload, state): (Json<CheckSuiteEvent>, State<WebState>)) -> impl Responder {
    state.process(WithInstallation {
        installation_id: InstallationId(payload.installation.id),
        sub_command: WithCommit {
            commit_id: payload.check_suite.head_sha.clone(), // TODO avoid clone
            sub_command: StartBuild {
                repo_url: payload.repository.url.to_string()
            }
        }
    }).unwrap_or_else(|e|
        error!("Failed to send start build command. {}", e));
    // TODO send better error back when failed to send command
    HttpResponse::Ok()
}
