use actix_web::*;
use actix_web::actix::*;
use actix_web::http::*;
use actix_web::pred::HeaderPredicate;
use hooks_api::check_suite::handle_check_suite;
use actix_web::middleware::Logger;
use model::state::ApplicationState;
use std::io;
use model::commands::with_app::Command;
use model::commands::with_app::WithApp;

pub struct WebState {
    app_actor: Addr<ApplicationState>
}

impl WebState {
    pub fn process<C>(&self, command: C) -> Result<(), SendError<WithApp<C>>>
        where C: Command + Send + 'static
    {
        self.app_actor.try_send(WithApp {
            sub_command: command
        })
    }
}

fn index(_req: &HttpRequest<WebState>) -> &'static str {
    "Hello world"
}

fn event_header_matches<S: 'static>(value: &'static str) -> HeaderPredicate<S> {
    pred::Header("X-GitHub-Event", value)
}

pub fn start_web_server(system_address: Addr<ApplicationState>) -> Result<(), io::Error> {
    server::new(move || {
        App::with_state(WebState { app_actor: system_address.clone() })
            .middleware(Logger::default())
            .resource("/", |r| r.f(index))
            .resource("/github/hook", |r| {
                r.method(Method::POST)
                    .filter(event_header_matches("check_suite"))
                    .with(handle_check_suite)
            },
            )
    }).bind("localhost:8080")?.run();
    Ok(())
}
