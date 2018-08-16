use actix_web::actix::*;
use model::state::ApplicationState;

impl Actor for ApplicationState {
    type Context = Context<Self>;
}

pub type Dispatcher = Addr<ApplicationState>;
