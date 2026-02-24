//! Job and scheduler DTOs for async add (MemOS scheduler/status compatibility).

use serde::{Deserialize, Serialize};

/// Status of an async add job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Pending,
    Running,
    Done,
    Failed,
}

/// A submitted or running add job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub job_id: String,
    pub status: JobStatus,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_summary: Option<serde_json::Value>,
}

/// Response when add is submitted in async mode (returns task_id for status polling).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddJobSubmitted {
    #[serde(default = "default_code")]
    pub code: i32,
    pub message: String,
    pub data: AddJobSubmittedData,
}

fn default_code() -> i32 {
    200
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddJobSubmittedData {
    pub task_id: String,
}

/// Response for GET /product/scheduler/status (MemOS-compatible).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerStatusResponse {
    #[serde(default = "default_code")]
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Job>,
}
