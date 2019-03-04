use reqwest::Client;
use reqwest::header;
use std::result;
use crate::hub::requests::GetCommitsRequest;
use crate::hub::responses::CommitsResponse;
use crate::hub::common::State;
use crate::hub::requests::SetStatusRequest;
use crate::hub::responses::StatusesResponse;

#[derive(Fail, Debug)]
pub enum GitHubError {
    #[fail(display = "{} header was not a valid", name)]
    InvalidHeader {
        name: &'static str,
    },

    #[fail(display = "HTTP error: {}", inner_error)]
    HttpError {
        inner_error: reqwest::Error,
    },
}

pub struct GitHubClient {
    client: Client,
}

pub type Result<T> = result::Result<T, GitHubError>;

const BASE_URL: &'static str = "https://api.github.com";

impl GitHubClient {
    pub fn new(token: &str) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        let auth_token = header::HeaderValue::from_str(&format!("token {}", token))
            .map_err(|_| GitHubError::InvalidHeader { name: header::AUTHORIZATION.as_str() })?;
        headers.insert(header::AUTHORIZATION, auth_token);
        let accept = header::HeaderValue::from_str("application/vnd.github.v3+json")
            .map_err(|_| GitHubError::InvalidHeader { name: header::ACCEPT.as_str() })?;
        headers.insert(header::ACCEPT, accept);

        Ok(GitHubClient {
            client: Client::builder()
                .default_headers(headers)
                .build().map_err(|inner_error| GitHubError::HttpError { inner_error })?
        })
    }

    pub fn get_last_commit<'a>(&self, repo: &'a RepoLocator, request: GetCommitsRequest)
                               -> Result<Option<CommitLocator<'a>>> {
        let commits_url = format!("{}/commits", &repo.url());
        let mut response = self.client.get(&commits_url)
            .json(&request)
            .send()
            .map_err(|inner_error| GitHubError::HttpError { inner_error })?;
        let commits: CommitsResponse = response.json()
            .map_err(|inner_error| GitHubError::HttpError { inner_error })?;
        let last_commit = commits.first().map(|c| CommitLocator {
            repo,
            sha: c.sha.to_string(),
        });
        Ok(last_commit)
    }

    pub fn get_statuses(&self, commit: &CommitLocator) -> Result<StatusesResponse> {
        let statuses_url = format!("{}/statuses/{}", &commit.repo.url(), &commit.sha);
        let mut response = self.client.get(&statuses_url)
            .send()
            .map_err(|inner_error| GitHubError::HttpError { inner_error })?;
        let statuses: StatusesResponse = response.json()
            .map_err(|inner_error| GitHubError::HttpError { inner_error })?;
        Ok(statuses)
    }

    pub fn set_status(&self, commit: &CommitLocator, request: SetStatusRequest) -> Result<()> {
        let statuses_url = format!("{}/statuses/{}", &commit.repo.url(), &commit.sha);
        self.client.post(&statuses_url)
            .json(&request)
            .send()
            .map_err(|inner_error| GitHubError::HttpError { inner_error })?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct RepoLocator {
    pub owner: String,
    pub repo: String,
}

impl RepoLocator {
    fn url(&self) -> String {
        format!("{}/repos/{}/{}", BASE_URL, &self.owner, &self.repo)
    }
}

#[derive(Debug)]
pub struct CommitLocator<'a> {
    repo: &'a RepoLocator,
    pub sha: String,
}

pub mod common {
    #[derive(Serialize, Deserialize, Debug)]
    pub enum State {
        #[serde(rename = "error")]
        Error,
        #[serde(rename = "failure")]
        Failure,
        #[serde(rename = "pending")]
        Pending,
        #[serde(rename = "success")]
        Success,
    }
}

pub mod requests {
    use crate::hub::common::State;

    #[derive(Serialize, Debug)]
    pub struct GetCommitsRequest<'a> {
        pub sha: &'a str
    }

    #[derive(Serialize, Debug)]
    pub struct SetStatusRequest<'a> {
        pub state: State,
        pub target_url: Option<&'a str>,
        pub description: Option<&'a str>,
        pub context: Option<&'a str>,
    }
}

pub mod responses {
    use crate::hub::common::State;

    pub type CommitsResponse = Vec<Commit>;

    #[derive(Deserialize, Debug)]
    pub struct Commit {
        pub sha: String,
        pub html_url: String,
    }

    pub type StatusesResponse = Vec<Status>;

    #[derive(Serialize, Deserialize, Debug)]
    pub struct Status {
        pub state: State,
        pub target_url: Option<String>,
        pub description: Option<String>,
        pub context: Option<String>,
    }
}
