use std::path::PathBuf;

use config::general_config::GeneralConfig;
use rocket::http::Status;
use rocket::State;

use crate::errors::{Error, ErrorKind};

fn get_general_config(path: &PathBuf) -> Result<GeneralConfig, Error> {
    GeneralConfig::read(&path).or_else(|e| Err(Error {
        kind: ErrorKind::NotFound,
        msg: String::from("Couldn't load general config from file"),
        cause: Some(Box::new(e)),
    }))
}

#[get("/profiles/config_dir")]
pub async fn get_profile_config_dir(profile_configs: &State<PathBuf>) -> (Status, String) {
    match get_general_config(profile_configs) {
        Ok(general_config) => (Status::Ok, format!("{:?}", general_config.profile_configs)),
        Err(e) => {
            log::error!("{}", e);
            match e.kind {
                ErrorKind::NotFound => (Status::NotFound, e.msg),
                _ => (Status::InternalServerError, String::from("Unkown error"))
            }
        },
    }
}
