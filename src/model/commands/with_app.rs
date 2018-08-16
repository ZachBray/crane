use actix_web::actix::*;
use futures::Future;
use model::state::ApplicationState;
use model::application::*;

pub struct AppContext<'a> {
    pub application: &'a mut ApplicationState,
    pub context: &'a mut Context<ApplicationState>,
}

impl<'a> AppContext<'a> {
    pub fn new(application: &'a mut ApplicationState,
               context: &'a mut Context<ApplicationState>) -> Self {
        AppContext {
            application,
            context,
        }
    }

    pub fn spawn<F>(&mut self, f: impl FnOnce(Dispatcher) -> F) -> ()
        where F: Future<Item=(), Error=()> + 'static
    {
        let future = f(self.context.address());
        Arbiter::spawn(future)
    }
}

pub trait Command {
    fn apply(self, context: AppContext) -> ();
}

pub struct WithApp<C> where C: Command {
    pub sub_command: C
}

impl<C> Message for WithApp<C> where C: Command {
    type Result = ();
}

impl<C> Handler<WithApp<C>> for ApplicationState where C: Command {
    type Result = ();

    fn handle(&mut self, msg: WithApp<C>, ctx: &mut Context<Self>) -> Self::Result {
        let system = AppContext::new(self, ctx);
        msg.sub_command.apply(system)
    }
}
