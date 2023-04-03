use std::path::PathBuf;

use api::get_profile_config_dir;
use cli_args::parse_args;

#[macro_use] extern crate rocket;

mod cli_args;
mod api;

#[get("/")]
async fn index() -> &'static str {
    "Hello"
}

#[launch]
fn rocket() -> _ {
    let args = parse_args();

    rocket::build()
        .manage(PathBuf::from(args.general_config))
        .mount("/", routes![index])
        .mount("/api", routes![get_profile_config_dir])
}
