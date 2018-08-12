extern crate actix_web;
extern crate serde;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate quick_error;

mod github_events;
mod crane_error;

use std::io;
use actix_web::*;
use actix_web::http::*;
use github_events::*;

fn index(_req: &HttpRequest) -> &'static str {
    "Hello world"
}

fn handle_hook(req: &HttpRequest) -> HttpResponse {
    let event_type = EventType::from_req(req);
    println!("Received: {:?}", event_type);
    HttpResponse::new(StatusCode::OK)
}

fn main() -> Result<(), io::Error> {
    server::new(|| {
        App::new().resource("/", |r| r.f(index)).resource(
            "/github/hook",
            |r| {
                r.method(Method::POST).f(handle_hook)
            },
        )
    }).bind("localhost:8080")?
        .run();
    Ok(())
}
