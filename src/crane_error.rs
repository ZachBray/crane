quick_error! {
    #[derive(Debug)]
    pub enum CraneError {
        MissingEventType {
            description("Missing event type.")
            display("Missing event type.")
        }
        InvalidEventType(event_type: String) {
            description("Invalid event type.")
            display("Event type '{}' is invalid.", event_type)
        }
    }
}
