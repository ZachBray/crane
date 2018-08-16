use actix_web::client;
use futures::future::*;
use model::commands::with_commit::*;
use model::state::CommitState;
use model::application::*;
use model::commands::with_app::WithApp;
use model::commands::with_installation::WithInstallation;
use github_client::commands::create_check::CreateCheck;
use github_client::commands::create_check::Check;
use github_client::commands::create_check::Output;
use github_client::commands::create_check::CheckCreated;

pub struct StartBuild {
    pub repo_url: String
}

impl StartBuild {
    fn create_github_check(self, context: &mut CommitContext) {
        let commit_id = context.commit_id.clone();
        context.parent.parent.application.github_client.try_send(CreateCheck {
            installation_id: *context.parent.installation_id,
            repo_url: self.repo_url,
            check: Check {
                name: "craneci",
                head_sha: commit_id.to_str().to_string(),
                output: Output {
                    title: "Crane CI",
                    summary: "TODO",
                },
            },
            reply_channel: |r| {
                match r {
                    Ok(CheckCreated { id }) => {
                        info!("Created check run id={}", id);
                    },
                    Err(e) => {
                        error!("Failed to create check run in GitHub: {}", e);
                    }
                }
            },
        }).unwrap_or_else(|e|
            error!("Failed to create check run in GitHub: {}", e));
    }
}


impl CommitCommand for StartBuild {
    fn apply_to_commit(self, mut context: CommitContext) -> Option<CommitState> {
        match context.commit {
            CommitState::Pending => {
                info!("Starting build");
                self.create_github_check(&mut context);
                Some(CommitState::Building {
                    agent_id: 0 // TODO assign best agent
                })
            }
            CommitState::Building { agent_id } => {
                info!("Already building on agent {}", agent_id);
                Some(context.commit)
            }
            CommitState::Built { was_successful } => {
                info!("Already built (successful: {})", was_successful);
                Some(context.commit)
            }
        }
    }
}