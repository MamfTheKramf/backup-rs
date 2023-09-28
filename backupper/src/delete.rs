use std::{fs, path::PathBuf};

use config::{general_config::GeneralConfig, profile_config::ProfileConfig};
use log::error;
use uuid::Uuid;

use crate::scheduler::{schedule_backup, unschedule_backup};

/// Deletes all the backup files belonging to the given [Uuid] within the given directory.
fn delete_backup_files(uuid: &Uuid, dir: &PathBuf) -> Result<(), String> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) => {
            return Err(format!("Error reading dir: {:?}", err));
        }
    };

    for entry in entries {
        if entry.is_err() {
            continue;
        }

        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            continue;
        }

        let filename = match path.file_name().and_then(|name| name.to_str()) {
            Some(name) => name,
            None => continue,
        };

        if filename.starts_with(&uuid.as_hyphenated().to_string()) && filename.ends_with(".zip") {
            if let Err(e) = fs::remove_file(&path) {
                error!("Couldn't delete {:?}. Got: {:#?}", path, e);
            }
        }

    }

    Ok(())
}

/// Deletes the config file for the given [ProfileConfig] and unschedules its backups.
/// If `delete_backups` is true, the profiles backups are alos deleted
pub fn delete(
    profile_config: &ProfileConfig,
    general_config: &GeneralConfig,
    delete_backups: bool,
) {
    if let Err(e) = unschedule_backup(profile_config.get_uuid().clone()) {
        error!("Couldn't unschedule profile. Got {}", e);
        return;
    }

    if delete_backups {
        if let Err(e) = delete_backup_files(profile_config.get_uuid(), &profile_config.target_dir) {
            error!("Couldn't delete previous backups. Got {}", e);

            if let Err(e) = schedule_backup(
                profile_config.get_uuid().clone(),
                profile_config.next_backup,
            ) {
                error!("Couldn't reschedule old backup. Got: {}", e);
            }
            return;
        }
    }

    let filename = profile_config.get_uuid().as_hyphenated().to_string() + ".json";
    let path = general_config.profile_configs.join(filename);

    if let Err(e) = fs::remove_file(&path) {
        error!("Coudln't delete config file. Got {:#?}", e);
        if let Err(e) = schedule_backup(
            profile_config.get_uuid().clone(),
            profile_config.next_backup,
        ) {
            error!("Couldn't reschedule old backup. Got: {}", e);
        }
    }
}
