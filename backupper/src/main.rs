mod common;
mod backup;
mod restore;
mod reschedule;
mod delete;
mod cli_args;
mod config;
mod dialog;
mod scheduler;

use std::{process::exit, path::PathBuf};

use backup::handle_profile;
use dialog::info_dialog;
use log::{info, error};
use reschedule::reschedule;
use restore::restore;

use crate::config::soft_load_profile_configs;

fn init_logger(path: &PathBuf) {
    match log4rs::init_file(path, Default::default()) {
        Ok(_) => info!("Initialized logger"),
        Err(e) => {
            eprintln!("Couldn't initialize logger: {:?}", e);
            std::process::exit(1);
        }
    }
}

fn main() {
    let orig_path = match std::env::current_dir() {
        Ok(path) => Some(path),
        Err(_) => None,
    };
    if let Ok(path) = std::env::current_exe() {
        if let Some(parent) = path.parent() {
            if let Err(err) = std::env::set_current_dir(parent.clone()) {
                error!("Couldn't change working dir beacuase {:?}", err);
                exit(1);
            }
        } else {
            error!("Couldn't change working directory.");
            exit(1);
        }
    }

    let args = cli_args::get_args();
    init_logger(&PathBuf::from(&args.logger_config));
    let general_config = match config::load_general_config(Some(&args.general_config)) {
        Ok(config) => config,
        Err(msg) => {
            error!("Error loading general config: {}", msg);
            exit(1);
        }
    };

    info!("Loaded general config: {:?}", general_config);

    let profile_configs = match soft_load_profile_configs(&general_config, &args) {
        Ok(configs) => configs,
        Err(msg) => {
            error!("Error loading profile configs: {}", msg);
            exit(1);
        }
    };

    info!("Loaded {} profile configs.", profile_configs.len());

    match &args.command {
        cli_args::Commands::Backup => {
            for mut profile_config in profile_configs {
                handle_profile(&mut profile_config, &general_config, &args)
            }
        
            info_dialog("Backup Abgeschlossen", "Das Backup ist abgeschlossen. Die externe Festplatte kann jetzt entfernt werden.");
        },
        cli_args::Commands::Restore(restore_params) => {
            let timestamp = restore_params.timestamp.unwrap_or_else(|| chrono::Local::now().naive_local());
            for profile_config in profile_configs {
                restore(&profile_config, timestamp, &args);
            }
        },
        cli_args::Commands::Reschedule => for mut profile_config in profile_configs {
            reschedule(&mut profile_config, &general_config);
        },
        cli_args::Commands::Delete(delete_params) => {
            for profile_config in profile_configs {
                delete::delete(&profile_config, &general_config, delete_params.remove_backups);
            }
        }
    }

    if let Some(orig_path) = orig_path {
        if let Err(err) = std::env::set_current_dir(orig_path) {
            error!("Couldn't reset working dir because of {:?}", err);
        }
    }
}
