use std::io;
use actix_web;
use serde_json;

quick_error! {
    #[derive(Debug)]
    pub enum Error {

        IOError(err: io::Error) {
            from()
            cause(err)
            description("I/O error")
            display("I/O error: {}", err)
        }

        RequestError(err: actix_web::client::SendRequestError) {
            from()
            description("Request error")
            display("Request error: {}", err)
        }

        PayloadError(err: actix_web::error::PayloadError) {
            from()
            description("Payload error")
            display("Payload error: {}", err)
        }

        ParseError(err: serde_json::Error) {
            from()
            cause(err)
            description("Parse error")
            display("Parse error: {}", err)
        }

        PrivateKeyLoadError {
            description("Failed to load private key.")
            display("Failed to load private key.")
        }
    }
}