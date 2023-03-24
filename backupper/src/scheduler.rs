//! Contains function for scheduling the execution of this binary for a certain datetime
//! 
//! # TODO
//! [] Scheduling on Linux

use chrono::NaiveDateTime;
use uuid::Uuid;

mod windows;

/// Schedules a backup for the profile with the given [Uuid] at the provided [NaiveDateTime].
///
/// # Errors
/// Returns an [Err] describing what went wrong if there was an issue.
pub fn schedule_backup(uuid: Uuid, date_time: NaiveDateTime) -> Result<(), String> {
    windows::schedule_backup(uuid, date_time)
}