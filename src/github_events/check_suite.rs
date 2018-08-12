
#[derive(Serialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum CheckSuiteAction {
    Requested,
    Rerequested,
    Completed
}

#[derive(Serialize, Debug)]
pub struct CheckSuite<'a> {
    head_sha: &'a str
}

#[derive(Serialize, Debug)]
pub struct CheckSuiteEvent<'a> {
    action: CheckSuiteAction,
    suite: CheckSuite<'a>
}

