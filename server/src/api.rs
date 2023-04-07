use std::ffi::OsStr;

use config::{general_config::GeneralConfig, profile_config::ProfileConfig};
use rocket::http::Status;
use rocket::State;
use rocket::tokio::fs;
use rocket::serde::{Serialize, json::Json};

#[allow(dead_code)]
type APIError = (Status, String);

/// Returns the path to the dir that stores the [ProfileConfig]s.
/// 
/// If the [GeneralConfig] file can't be opened, a `404` Error is returned.
/// On other errors, a `500` Error is returned.
#[get("/profiles/config_dir")]
pub async fn get_profile_config_dir(general_config: &State<GeneralConfig>) -> (Status, String) {
    (Status::Ok, format!("{:?}", general_config.profile_configs))
}

/// Contains protobuf serializes [ProfileConfig]s
#[derive(Debug, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct ProfileConfigs {
    pub configs: Vec<Vec<u8>>,
}

/// Returns a vector of the [ProfileConfig]s that are found inside te directory specified in the [GeneralConfig]
#[get("/profiles")]
pub async fn get_profile_configs(general_config: &State<GeneralConfig>) -> Result<(Status, Json<Vec<ProfileConfig>>), APIError> {
    let dir = &general_config.profile_configs;

    let fs_err_msg = "Couldn't open directory with profile configs";

    let mut dir = fs::read_dir(dir)
        .await
        .or_else(|e| {
            log::error!("Couldn't read profile-configs dir because {:#?}", e);
            Err((Status::NotFound, String::from(fs_err_msg)))
        })?;

    let mut profile_configs = vec![];

    while let Some(entry) = dir.next_entry()
            .await
            .or_else(|e| {
                log::error!("{:#?}", e);
                Err((Status::NotFound, String::from(fs_err_msg)))
            })? 
    {
        let path = entry.path();

        if &path.extension().unwrap_or(OsStr::new("")).to_str().unwrap_or("") != &"json" {
            log::debug!("Skip entry {:?} because it's not a JSON file", &path);
            continue;
        }

        if !&path.is_file() {
            log::debug!("Skip entry {:?} because it's not a file", &path);
            continue;
        }

        match ProfileConfig::load(&path) {
            Ok(profile) => profile_configs.push(profile),
            Err(e) => {
                log::debug!("Couldn't load profile config {:?} because {:#?}", &path, e);
                continue;
            }
        }
    }

    Ok((Status::Ok, Json(profile_configs)))
}
