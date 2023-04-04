use std::path::PathBuf;

use config::general_config::GeneralConfig;
use rocket::http::Status;
use rocket::State;

use crate::errors::{Error, ErrorType};

fn get_general_config(path: &PathBuf) -> Result<GeneralConfig, Error> {
    GeneralConfig::read(&path).or(Err(Error {
        kind: ErrorType::NotFound,
        msg: String::from("Couldn't load general config from file"),
        cause: None,
    }))
}

#[get("/profiles/config_dir")]
pub async fn get_profile_config_dir(profile_configs: &State<PathBuf>) -> (Status, String) {
    match get_general_config(profile_configs) {
        Ok(general_config) => (Status::Ok, format!("{:?}", general_config.profile_configs)),
        Err(e) => ,
    }
}
