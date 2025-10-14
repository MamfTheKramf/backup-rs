use config::{general_config::GeneralConfig, profile_config::ProfileConfig};
use log::{error, info};

use crate::scheduler;

/// Reschedules the backup of the given [ProfileConfig].
/// This usually happens, when the [Interval] changes.
///
/// The `next_backup` field of the `profile_config` will be set to the next match after today.
pub fn reschedule(profile_config: &mut ProfileConfig, general_config: &GeneralConfig) {
    profile_config.update_next_backup();

    if let Err(e) =
        scheduler::schedule_backup(*profile_config.get_uuid(), profile_config.next_backup)
    {
        error!("Couldn't schedule next backup: {:?}", e);
        return;
    }

    if let Err(e) = profile_config.store(&general_config.profile_configs) {
        error!("Couldn't store updated ProfileConfig: {:?}", e);
        info!("The backup is still rescheduled though.");
    }
}
