use config::{profile_config::ProfileConfig, general_config::GeneralConfig};

use crate::scheduler;



/// Reschedules the backup of the given [ProfileConfig].
/// This usually happens, when the [Interval] changes.
/// 
/// The `next_backup` field of the `profile_config` will be set to the next match after today.
pub fn reschedule(profile_config: &mut ProfileConfig, general_config: &GeneralConfig) {
    let now = chrono::Local::now().naive_local();

    let next_backup = profile_config.get_next_scheduled(Some(now));
    profile_config.next_backup = next_backup;

    if let Err(e) = scheduler::schedule_backup(profile_config.get_uuid().clone(), profile_config.next_backup) {
        println!("Couldn't schedule next backup: {:?}", e);
        return;
    }

    if let Err(e) = profile_config.store(&general_config.profile_configs) {
        println!("Couldn't store updated ProfileConfig: {:?}", e);
        println!("The backup is still rescheduled though.");
    }
}