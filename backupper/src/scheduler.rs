//! Contains function for scheduling the execution of this binary for a certain datetime
//!
//! # TODO
//! [] Scheduling on Linux

use chrono::NaiveDateTime;
use log::info;
use uuid::Uuid;

#[cfg(target_family = "windows")]
mod windows;

/// Schedules a backup for the profile with the given [Uuid] at the provided [NaiveDateTime].
///
/// # Errors
/// Returns an [Err] describing what went wrong if there was an issue.
pub fn schedule_backup(uuid: Uuid, date_time: NaiveDateTime) -> Result<(), String> {
    info!("Schedule backup call for {} at {}", uuid, date_time);
    #[cfg(target_family = "windows")]
    windows::schedule_backup(uuid, date_time)
}

/// Unschedules backups for the profile with the given [Uuid]
pub fn unschedule_backup(uuid: Uuid) -> Result<(), String> {
    info!("Unschedule backup call for {}", uuid);
    #[cfg(target_family = "windows")]
    windows::unschedule_backup(uuid)
}
