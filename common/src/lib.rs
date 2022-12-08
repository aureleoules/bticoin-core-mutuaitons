use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

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

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Mutation {
    pub id: String,
    pub file: String,
    pub line: usize,
    pub patch: String,
    pub branch: String,
    pub pr_number: Option<String>,
    pub status: MutationStatus,
    #[serde(default, with = "time::serde::timestamp::option")]
    pub start_time: Option<OffsetDateTime>,
    #[serde(default, with = "time::serde::timestamp::option")]
    pub end_time: Option<OffsetDateTime>,
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
