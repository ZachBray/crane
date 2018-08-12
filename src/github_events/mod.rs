use actix_web::HttpRequest;
use super::crane_error::CraneError;

pub mod check_suite;

#[derive(Debug)]
pub enum EventType {
    CheckSuite,
}

impl EventType {

    pub fn from_req(req: &HttpRequest) -> Result<EventType, CraneError> {
        let header_value = req.headers()
            .get("X-GitHub-Event")
            .ok_or(CraneError::MissingEventType)?;
        if header_value == "check_suite" {
            Ok(EventType::CheckSuite)
        } else {
            Err(CraneError::InvalidEventType("TODO".to_owned()))
        }
    }
}
