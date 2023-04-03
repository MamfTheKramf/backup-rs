use std::path::PathBuf;


use rocket::State;
use rocket::http::Status;



#[get("/profiles/config_dir")]
pub async fn get_profile_config_dir(profile_configs: &State<PathBuf>) -> (Status, String) {
    (Status::Ok, profile_configs.to_str().unwrap().to_string())
}