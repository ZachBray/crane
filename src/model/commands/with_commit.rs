use model::state::CommitState;
use model::state::InstallationState;
use model::values::sha::Sha;
use model::commands::with_installation::*;

pub struct CommitContext<'a: 'b, 'b: 'c, 'c> {
    pub commit_id: &'c Sha,
    pub commit: CommitState,
    pub parent: &'c mut InstallationContext<'a, 'b>
}

pub trait CommitCommand {
    fn apply_to_commit(self, context: CommitContext) -> Option<CommitState>;
}

pub struct WithCommit<C> where C: CommitCommand {
    pub commit_id: Sha,
    pub sub_command: C,
}

impl<C> InstallationCommand for WithCommit<C> where C: CommitCommand {
    fn apply_to_installation(self, mut context: InstallationContext) -> Option<InstallationState>
    {
        let id = self.commit_id.clone();
        let commit = context.installation.commits.remove(&id)
            .unwrap_or_else(|| CommitState::Pending);
        let updated_commit_maybe = {
            let commit_context = CommitContext {
                commit_id: &id,
                commit,
                parent: &mut context
            };
            self.sub_command.apply_to_commit(commit_context)
        };
        if let Some(updated_commit) =  updated_commit_maybe {
            context.installation.commits.insert(id, updated_commit);
        }
        Some(context.installation)
    }
}


