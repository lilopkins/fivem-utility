#![deny(missing_docs)]

//! A crate of utility functions for FiveM, including function related to
//! versioning and config management. The `config` module contains
//! parsers for the `.cfg` files. The `artifacts` module contains functions
//! to download a list of artifacts available from the artifact server.

extern crate clap;
extern crate colored;

use clap::{Arg, App, SubCommand};
use colored::*;

use std::fs;
use std::process::exit;
use std::collections::HashMap;

/// The config module contains functions for parsing and making sense of `.cfg`
/// files that servers use to start.
pub mod config;
/// The artifacts module contains functions for fetching information about available
/// artifacts from the artifact server.
pub mod artifacts;

/// A function to detect resources within a resources folder.
pub fn detect_resources(resource_dir: &str) -> HashMap<String, String> {
    let paths = fs::read_dir(resource_dir).unwrap();
    let mut resources: HashMap<String, String> = HashMap::new();

    for path in paths {
        let entry = path.unwrap();
        let meta = entry.metadata().unwrap();
        if meta.is_dir() {
            let name = entry.file_name().into_string().unwrap();
            let file_path = entry.path().into_os_string().into_string().unwrap();
            if name.starts_with("[") && name.ends_with("]") {
                // recurse downwards
                let sub_resources = detect_resources(&file_path);
                for key in sub_resources.keys() {
                    resources.insert(key.clone(), sub_resources[key].clone());
                }
            } else {
                resources.insert(name, file_path);
            }
        }
    }

    resources
}

fn main() {
    let matches = App::new("fivem-utility")
                    .version(env!("CARGO_PKG_VERSION"))
                    .author("Lily H <bsalarius@gmail.com>")
                    .about("Provides various useful utilities for FiveM servers")
                    .arg(Arg::with_name("config")
                            .short("c")
                            .long("config")
                            .help("Set the main config file, often called `server.cfg'")
                            .takes_value(true))
                    .arg(Arg::with_name("resources-dir")
                            .short("r")
                            .long("resources-dir")
                            .help("Set the resources directory")
                            .takes_value(true))
                    .subcommand(SubCommand::with_name("print")
                            .about("Print details about the config file."))
                    .subcommand(SubCommand::with_name("verify")
                            .about("Checks the integrity of the config file."))
                    .subcommand(SubCommand::with_name("resource-usage")
                            .about("Finds resources specified in server.cfg, and lists resources that are never used."))
                    .subcommand(SubCommand::with_name("version-server")
                            .about("Gives information about the versions available from the FiveM version server")
                            .arg(Arg::with_name("use-windows-server")
                                .short("w")
                                .help("Use the Windows artifact server (default's to the linux artifact server)")))
                    .get_matches();

    let config_file = matches.value_of("config").unwrap_or("server.cfg");
    let resources_dir = matches.value_of("resources-dir").unwrap_or("resources");

    if let Some(_) = matches.subcommand_matches("print") {
        let cfg = config::read_config_file(config_file).ok().unwrap_or_else(|| {
            panic!("Failed to parse config file. Maybe run `verify` to check why?");
        });
        cfg.print_nicely();
    } else if let Some(_) = matches.subcommand_matches("verify") {
        let config_result = config::read_config_file(config_file);
        if config_result.is_ok() {
            eprintln!("The file was parsed and found no errors.");
            exit(0);
        } else {
            eprintln!("The file was parsed and error(s) were found: {}", config_result.err().unwrap());
            exit(1);
        }
    } else if let Some(_) = matches.subcommand_matches("resource-usage") {
        let cfg = config::read_config_file(config_file).ok().unwrap_or_else(|| {
            panic!("Failed to parse config file. Maybe run `verify` to check why?");
        });
        let mut resources = detect_resources(resources_dir);
        for res in cfg.resources {
            let found = resources.remove(&res);
            match found {
                Some(val) => println!("{} {} @ {}", "[  FOUND  ]".green(), res.bold(), val),
                None => eprintln!("{} {}", "[ MISSING ]".red(), res.bold()),
            };
        }
        for key in resources.keys() {
            eprintln!("{} {} @ {}", "[  EXTRA  ]".yellow(), key.bold(), &resources[key]);
        }
    } else if let Some(matches) = matches.subcommand_matches("version-server") {
        let mut url = "https://runtime.fivem.net/artifacts/fivem/build_proot_linux/master/";

        if cfg!(target_os = "windows")
            || matches.is_present("use-windows-server") {
            
            url = "https://runtime.fivem.net/artifacts/fivem/build_server_windows/master/";
        }

        let afs = artifacts::get_artifacts(url.to_string());
        for af in afs {
            println!("{}\t{}", af.num, af.url);
        }
    } else {
        eprintln!("You must specify a subcommand. See --help for more information.");
        exit(1);
    }
}
