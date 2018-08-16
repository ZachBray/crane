use model::state::*;
use model::values::installation_id::InstallationId;
use model::commands::with_app::*;

pub struct InstallationContext<'a: 'b, 'b> {
    pub installation_id: &'b InstallationId,
    pub installation: InstallationState,
    pub parent: &'b mut AppContext<'a>
}

pub trait InstallationCommand {
    fn apply_to_installation(self, context: InstallationContext) -> Option<InstallationState>;
}

pub struct WithInstallation<C> where C: InstallationCommand {
    pub installation_id: InstallationId,
    pub sub_command: C,
}

impl<C> Command for WithInstallation<C> where C: InstallationCommand {
    fn apply(self, mut context: AppContext) -> () {
        let id = self.installation_id.clone();
        let installation = {
            context.application.installations
                .remove(&id)
                .unwrap_or_else(|| InstallationState::empty())
        };
        let updated_installation_maybe = {
            let installation_context = InstallationContext {
                installation_id: &self.installation_id,
                installation,
                parent: &mut context
            };
            self.sub_command.apply_to_installation(installation_context)
        };
        if let Some(updated_installation) = updated_installation_maybe {
            context.application.installations.insert(id, updated_installation);
        }
    }
}