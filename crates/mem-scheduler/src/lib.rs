//! Scheduler for async add: submit_add returns job_id, worker runs add, get_status polls.

mod memory;
mod trait_;

pub use memory::InMemoryScheduler;
pub use trait_::{Scheduler, SchedulerError};
