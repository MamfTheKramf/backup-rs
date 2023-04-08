use std::ffi::OsStr;
use std::path::PathBuf;

use config::{general_config::GeneralConfig, profile_config::ProfileConfig};
use rocket::http::Status;
use rocket::State;
use rocket::tokio::fs;
use rocket::serde::{Serialize, json::Json};

use crate::errors::{Error, ErrorKind};

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

/// Loads all the profile configs from the provided directory and returns them
async fn read_profile_configs(path: &PathBuf) -> Result<Vec<ProfileConfig>, Error> {
    let mut dir = fs::read_dir(path)
        .await
        .or_else(|e| {
            log::error!("Couldn't read profile-configs dir because {:#?}", e);
            Err(Error {
                kind: ErrorKind::Internal,
                msg: String::from("Couldn't open directory with profile configs"),
                cause: Some(Box::new(e))
            })
        })?;

    let mut profile_configs = vec![];

    // go through each entry, check that it's a JSON and try to desrialize it
    while let Some(entry) = dir.next_entry()
            .await
            .or_else(|e| {
                log::error!("Couldn't get netx dir entry because {:#?}", e);
                Err(Error {
                    kind: ErrorKind::Internal,
                    msg: String::from("Couldn't read directory entry"),
                    cause: Some(Box::new(e))
                })
            })? 
    {
        let path = entry.path();

        // check that it's a JSON
        if &path.extension().unwrap_or(OsStr::new("")) != &"json" {
            log::debug!("Skip entry {:?} because it's not a JSON file", &path);
            continue;
        }

        // check that it's a file and exist
        if !&path.is_file() {
            log::debug!("Skip entry {:?} because it's not a file", &path);
            continue;
        }

        // try parsing
        match ProfileConfig::load(&path) {
            Ok(profile) => profile_configs.push(profile),
            Err(e) => {
                log::debug!("Couldn't load profile config {:?} because {:#?}", &path, e);
                continue;
            }
        }
    }

    Ok(profile_configs)
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

    let profile_configs = read_profile_configs(dir)
        .await
        .or_else(|e| Err((Status::InternalServerError, e.msg)))?;

    Ok((Status::Ok, Json(profile_configs)))
}
