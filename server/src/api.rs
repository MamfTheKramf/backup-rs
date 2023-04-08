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

/// Tries to delete the JSON file of that profile config
async fn delete_profile_config(
    json_dir: &PathBuf,
    profile_config: ProfileConfig,
) -> Result<(), APIError> {
    let filename = profile_config.get_uuid().as_hyphenated().to_string() + ".json";
    let path = json_dir.join(filename);

    fs::remove_file(&path).await.or_else(|e| {
        log::error!("Couldn't delete file {:?} because {:#?}", path, e);
        match e.kind() {
            std::io::ErrorKind::NotFound => Ok(()),
            _ => Err((
                Status::InternalServerError,
                format!(
                    "Couldn't remove ProfileConfig with name {:?}",
                    profile_config.name
                ),
            )),
        }
    })
}

/// Deletes the [ProfileConfig] with the given name.
#[delete("/profiles/name/<name>")]
pub async fn delete_profile_config_by_name(
    general_config: &State<GeneralConfig>,
    name: String,
) -> Result<Status, APIError> {
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

    delete_profile_config(dir, target_config).await?;

    Ok(Status::NoContent)
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

    let name_already_taken = profile_configs.iter().any(|config| config.name == name);
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
