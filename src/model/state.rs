use fnv::FnvHashMap;
use model::values::sha::Sha;
use model::values::installation_id::InstallationId;
use github_client::github_client::GitHubClient;

pub enum CommitState {
    Pending,
    Building {
        agent_id: u32,
    },
    Built {
        was_successful: bool,
    }
}

pub struct AgentState {
    pub ttl: u64,
    pub idle_probability: f32,
}

pub struct InstallationState {
    // TODO only keep N builds + agents
    pub commits: FnvHashMap<Sha, CommitState>,
    pub agents: FnvHashMap<u32, AgentState>,
}

impl InstallationState {
    pub fn empty() -> Self {
        InstallationState {
            commits: FnvHashMap::default(),
            agents: FnvHashMap::default(),
        }
    }
}

pub struct ApplicationState {
    pub time: u64,
    pub installations: FnvHashMap<InstallationId, InstallationState>,
    pub github_client: GitHubClient,
}

impl ApplicationState {
    pub fn new(github_client: GitHubClient) -> Self {
        ApplicationState {
            time: 0,
            installations: FnvHashMap::default(),
            github_client
        }
    }
}
