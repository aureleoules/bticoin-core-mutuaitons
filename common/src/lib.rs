use serde::{Deserialize, Serialize};


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum MutationStatus {
    Pending,
    Timeout,
    Running,
    Killed,
    NotKilled,
    Ignored,
    Error,
}

// to string
impl std::fmt::Display for MutationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MutationStatus::Pending => write!(f, "Pending"),
            MutationStatus::Timeout => write!(f, "Timeout"),
            MutationStatus::Running => write!(f, "Running"),
            MutationStatus::Killed => write!(f, "Killed"),
            MutationStatus::NotKilled => write!(f, "NotKilled"),
            MutationStatus::Ignored => write!(f, "Ignored"),
            MutationStatus::Error => write!(f, "Error"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Mutation {
    pub id: i64,
    pub patch_md5: Option<String>,
    pub file: Option<String>,
    pub line: Option<i64>,
    pub patch: Option<String>,
    pub branch: Option<String>,
    pub pr_number: Option<i64>,
    pub status: Option<String>,
    pub start_time: Option<i64>,
    pub end_time: Option<i64>,
    pub stderr: Option<String>,
    pub stdout: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MutationResult {
    pub mutation_id: String,
    pub status: MutationStatus,
    pub stdout: Option<String>,
    pub stderr: Option<String>,
}
