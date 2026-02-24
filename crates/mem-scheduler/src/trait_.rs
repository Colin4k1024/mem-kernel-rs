//! Scheduler trait: submit add job, get status.

use async_trait::async_trait;
use mem_types::{ApiAddRequest, Job};

#[derive(Debug, thiserror::Error)]
pub enum SchedulerError {
    #[error("scheduler error: {0}")]
    Other(String),
    #[error("job not found: {0}")]
    JobNotFound(String),
}

/// Scheduler for async add: submit returns job_id, status can be polled.
#[async_trait]
pub trait Scheduler: Send + Sync {
    /// Submit an add request; returns job_id. When async, the actual add runs in a worker.
    async fn submit_add(&self, req: ApiAddRequest) -> Result<String, SchedulerError>;

    /// Get current job status by job_id (task_id).
    async fn get_status(&self, job_id: &str) -> Result<Option<Job>, SchedulerError>;
}
