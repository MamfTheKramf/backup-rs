use std::ffi::OsStr;
use std::path::PathBuf;

use config::interval::IntervalBuilder;
use config::{general_config::GeneralConfig, profile_config::ProfileConfig};
use rocket::http::Status;
use rocket::serde::{json::Json, Serialize};
use rocket::tokio::fs;
use rocket::State;
use uuid::Uuid;

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
    let mut dir = fs::read_dir(path).await.or_else(|e| {
        log::error!("Couldn't read profile-configs dir because {:#?}", e);
        Err(Error {
            kind: ErrorKind::Internal,
            msg: String::from("Couldn't open directory with profile configs"),
            cause: Some(Box::new(e)),
        })
    })?;

    let mut profile_configs = vec![];

    // go through each entry, check that it's a JSON and try to desrialize it
    while let Some(entry) = dir.next_entry().await.or_else(|e| {
        log::error!("Couldn't get netx dir entry because {:#?}", e);
        Err(Error {
            kind: ErrorKind::Internal,
            msg: String::from("Couldn't read directory entry"),
            cause: Some(Box::new(e)),
        })
    })? {
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
pub async fn get_profile_configs(
    general_config: &State<GeneralConfig>,
) -> Result<(Status, Json<Vec<ProfileConfig>>), APIError> {
    let dir = &general_config.profile_configs;

    let profile_configs = read_profile_configs(dir)
        .await
        .or_else(|e| Err((Status::InternalServerError, e.msg)))?;

    Ok((Status::Ok, Json(profile_configs)))
}

/// Returns the [ProfileConfig] with the specified `name` or a `404` if it doesn't exist
#[get("/profiles/name/<name>")]
pub async fn get_profile_config_by_name(
    general_config: &State<GeneralConfig>,
    name: String,
) -> Result<(Status, Json<ProfileConfig>), APIError> {
    let dir = &general_config.profile_configs;

    let profile_configs = read_profile_configs(dir)
        .await
        .or_else(|e| Err((Status::InternalServerError, e.msg)))?;

    let target_config = profile_configs
        .into_iter()
        .find(|config| config.name == name)
        .ok_or_else(|| {
            let msg = format!("No ProfileConfig with the name {:?} was found", name);
            log::warn!("{}", msg);
            (Status::NotFound, msg)
        })?;

    Ok((Status::Ok, Json(target_config)))
}

#[derive(Debug)]
enum Identifyier {
    Name(String),
    Uuid(Uuid),
}

/// Tries to delete the JSON file of that profile config
async fn delete_profile_config(
    backupper_path: &PathBuf,
    id: Identifyier
) -> Result<String, String> {
    let mut output = rocket::tokio::process::Command::new(backupper_path.as_os_str());

    match &id {
        Identifyier::Name(name) => {
            output.arg("-n")
                .arg(name);
        }
        Identifyier::Uuid(uuid) => {
            output.arg("-u")
                .arg(uuid.as_hyphenated().to_string());
        },
    }

    let output = output.arg("delete")
        .output()
        .await;
    log::debug!("{:#?}", output);
    match output {
        Ok(output) => match output.status.code() {
            Some(x) if x == 0 => Ok(format!("Successfully deleted ProfileConfig {:?}", id)),
            _ => Err(format!(
                "Deletion of ProfileConfig {:?} failed. Error: {:#?}",
                id, output
            )),
        },
        Err(e) => Err(format!(
            "Deletion of ProfileConfig {:?} failed. Error: {:#?}",
            id, e
        )),
    }
}

/// Deletes the [ProfileConfig] with the given name.
#[delete("/profiles/name/<name>")]
pub async fn delete_profile_config_by_name(
    backupper_path: &State<PathBuf>,
    name: String,
) -> Result<Status, APIError> {
    log::debug!("Delete ProfileConfig {:?}", name);
    let res = delete_profile_config(backupper_path, Identifyier::Name(name)).await;

    match res {
        Ok(msg) => {
            log::debug!("{}", msg);
            Ok(Status::NoContent)
        },
        Err(msg) => {
            log::warn!("{}", msg);
            Err((Status::InternalServerError, msg))
        }
    }
}

#[get("/profiles/uuid/<uuid>")]
pub async fn get_profile_config_by_uuid(
    general_config: &State<GeneralConfig>,
    uuid: String,
) -> Result<(Status, Json<ProfileConfig>), APIError> {
    let uuid = Uuid::parse_str(&uuid).or_else(|e| {
        log::warn!("Couldn't parse uuid {:?} because {:#?}", uuid, e);
        Err((
            Status::BadRequest,
            format!("{:?} is not a valid uuid", uuid),
        ))
    })?;

    let dir = &general_config.profile_configs;

    let profile_configs = read_profile_configs(dir)
        .await
        .or_else(|e| Err((Status::InternalServerError, e.msg)))?;

    let target_config = profile_configs
        .into_iter()
        .find(|config| config.get_uuid() == &uuid)
        .ok_or_else(|| {
            let msg = format!("No ProfileConfig with the uuid {:?} was found", uuid);
            log::warn!("{}", msg);
            (Status::NotFound, msg)
        })?;

    Ok((Status::Ok, Json(target_config)))
}

// Deletes [ProfileConfig] with the given uuid
#[delete("/profiles/uuid/<uuid>")]
pub async fn delete_profile_config_by_uuid(
    backupper_path: &State<PathBuf>,
    uuid: String,
) -> Result<Status, APIError> {
    let uuid = Uuid::parse_str(&uuid).or_else(|e| {
        log::warn!("Couldn't parse uuid {:?} because {:#?}", uuid, e);
        Err((
            Status::BadRequest,
            format!("{:?} is not a valid uuid", uuid),
        ))
    })?;

    log::info!("Delete ProfileConfig {:?}", uuid);
    let msg = delete_profile_config(backupper_path, Identifyier::Uuid(uuid)).await;

    match msg {
        Ok(msg) => {
            log::debug!("{}", msg);
            Ok(Status::NoContent)
        },
        Err(msg) => {
            log::warn!("{}", msg);
            Err((Status::InternalServerError, msg))
        }
    }
}

/// Tries to create a new profile config with the given name.
///
/// # Returns
/// On success: The created profile config
/// On error: An error describing the issue
#[post("/profiles/create/<name>")]
pub async fn create_blank_profile_config(
    general_config: &State<GeneralConfig>,
    name: String,
) -> Result<(Status, Json<ProfileConfig>), APIError> {
    let dir = &general_config.profile_configs;

    let profile_configs = read_profile_configs(dir)
        .await
        .or_else(|e| Err((Status::InternalServerError, e.msg)))?;

    let name_already_taken = profile_configs
        .iter()
        .any(|config| config.name.to_lowercase() == name.to_lowercase());
    if name_already_taken {
        return Err((
            Status::Conflict,
            format!("Name {:?} is already taken", name),
        ));
    }

    let none_interval = IntervalBuilder::default()
        .minutes(config::interval::SpecifierKind::None)
        .build()
        .or_else(|e| {
            log::error!("Couldn't build none_interval because {:#?}", e);
            Err((
                Status::InternalServerError,
                String::from("Unexpected Error"),
            ))
        })?;
    let profile_config = ProfileConfig::new(
        name,
        PathBuf::from(""),
        vec![],
        vec![],
        vec![],
        vec![],
        none_interval,
    );

    profile_config.store(dir).or_else(|e| {
        log::error!("Couldn't store new ProfileConfig because {:#?}", e);
        Err((
            Status::InternalServerError,
            String::from("Unexpected Error"),
        ))
    })?;

    Ok((Status::Created, Json(profile_config)))
}

/// Updates the [ProfileConfig] with the given `uuid`. The `uuid` and the `next-backup` field won't be considered for updating
#[put("/profiles/uuid/<uuid>", data = "<new_config>")]
pub async fn update_profile_config(
    general_config: &State<GeneralConfig>,
    backupper_path: &State<PathBuf>,
    uuid: String,
    new_config: Json<ProfileConfig>,
) -> Result<(Status, Json<ProfileConfig>), APIError> {
    let uuid = Uuid::parse_str(&uuid).or_else(|e| {
        log::warn!("Couldn't parse uuid {:?} because {:#?}", uuid, e);
        Err((
            Status::BadRequest,
            format!("{:?} is not a valid uuid", uuid),
        ))
    })?;

    if let Err(msg) = new_config.interval.validate() {
        log::warn!("Got invalid interval: {:?}", msg);
        return Err((Status::BadRequest, msg));
    }

    let dir = &general_config.profile_configs;

    let profile_configs = read_profile_configs(dir)
        .await
        .or_else(|e| Err((Status::InternalServerError, e.msg)))?;

    let mut new_config = new_config.0;

    // check that name isn't already taken
    let name_already_taken = profile_configs.iter().any(|config| {
        config.get_uuid() != new_config.get_uuid()
            && config.name.to_lowercase() == new_config.name.to_lowercase()
    });
    if name_already_taken {
        return Err((
            Status::Conflict,
            format!("Name {:?} is already taken", new_config.name),
        ));
    }

    let target_config = profile_configs
        .into_iter()
        .find(|config| config.get_uuid() == &uuid)
        .ok_or_else(|| {
            let msg = format!("No ProfileConfig with the uuid {:?} was found", uuid);
            log::warn!("{}", msg);
            (Status::NotFound, msg)
        })?;

    new_config.set_uuid(uuid);
    new_config.next_backup = target_config.next_backup;

    // we have to store first; otherwise the reschedule would just take the old interval
    new_config.store(dir).or_else(|e| {
        log::error!(
            "Couldn't store ProfileConfig {:?} because {:#?}",
            new_config.get_uuid(),
            e
        );
        Err((
            Status::InternalServerError,
            String::from("Unexpected Error"),
        ))
    })?;

    if new_config.interval != target_config.interval {
        log::info!("Rescheduling ProfileConfig {:?}", new_config.get_uuid());
        let output = rocket::tokio::process::Command::new(backupper_path.as_os_str())
            .arg("-u")
            .arg(new_config.get_uuid().to_string())
            .arg("reschedule")
            .output()
            .await;
        log::debug!("{:#?}", output);
        match output {
            Ok(output) => match output.status.code() {
                Some(x) if x == 0 => log::debug!(
                    "Rescheduling of ProfileConfig {:?} successful",
                    new_config.get_uuid()
                ),
                _ => log::warn!(
                    "Rescheduling of ProfileConfig {:?} failed. Error: {:#?}",
                    new_config.get_uuid(),
                    output
                ),
            },
            Err(e) => log::warn!(
                "Rescheduling of ProfileConfig {:?} failed. Error: {:#?}",
                new_config.get_uuid(),
                e
            ),
        }
    }

    Ok((Status::Ok, Json(new_config)))
}
