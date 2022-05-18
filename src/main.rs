#![deny(missing_docs)]

//! A crate of utility functions for FiveM, including function related to
//! versioning and config management. The `config` module contains
//! parsers for the `.cfg` files. The `artifacts` module contains functions
//! to download a list of artifacts available from the artifact server.

use clap::{Parser, Subcommand};
use colored::*;

use std::collections::HashMap;
use std::fs;
use std::process::exit;

/// The artifacts module contains functions for fetching information about available
/// artifacts from the artifact server.
pub mod artifacts;
/// The config module contains functions for parsing and making sense of `.cfg`
/// files that servers use to start.
pub mod config;

/// A function to detect resources within a resources folder.
pub fn detect_resources(resource_dir: &str) -> HashMap<String, String> {
    let paths = fs::read_dir(resource_dir).unwrap();
    let mut resources: HashMap<String, String> = HashMap::new();

    for path in paths {
        let entry = path.unwrap();
        let meta = entry.metadata().unwrap();
        if meta.is_dir() {
            let name = entry.file_name().into_string().unwrap();
            let file_path = entry.path().to_str().unwrap().to_owned();
            if name.starts_with("[") && name.ends_with("]") {
                // recurse downwards
                let sub_resources = detect_resources(&file_path);
                for key in sub_resources.keys() {
                    resources.insert(key.clone(), sub_resources[key].clone());
                }
            } else {
                resources.insert(name, file_path);
            }
        } else {
            // Check if symlink
            let poss_symlink = fs::metadata(entry.path()).unwrap();
            if poss_symlink.is_dir() {
                // Is symlink to dir
                let sub_resources = detect_resources(&entry.path().to_str().unwrap().to_owned());
                for key in sub_resources.keys() {
                    resources.insert(key.clone(), sub_resources[key].clone());
                }
            }
        }
    }

    resources
}

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Set the main config file
    #[clap(short, long, default_value = "server.cfg")]
    config: String,

    /// Set the resources directory
    #[clap(short, long, default_value = "resources")]
    resources_dir: String,

    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Subcommand, Debug)]
enum SubCommand {
    /// Print details about the config file.
    Print,
    /// Checks the integrity of the config file.
    Verify,
    /// Finds resources specified in server.cfg, and lists resources that are never used.
    ResourceUsage,
    /// Gives information about the versions available from the FiveM version server.
    VersionServer(VersionServerArgs),
}

#[derive(Parser, Debug)]
struct VersionServerArgs {
    /// Get the URL of a server download from the version server
    #[clap(short, long)]
    get_url: Option<String>,

    /// Use the Windows artifact server instead of the Linux one
    #[clap(short = 'w', long)]
    use_windows_server: bool,
}

fn main() {
    let args = Args::parse();

    match args.subcmd {
        SubCommand::Print => {
            let cfg = config::read_config_file(&args.config)
                .ok()
                .unwrap_or_else(|| {
                    panic!("Failed to parse config file. Maybe run `verify` to check why?");
                });
            cfg.print_nicely();
        }
        SubCommand::Verify => {
            let config_result = config::read_config_file(&args.config);
            if config_result.is_ok() {
                eprintln!("The file was parsed and found no errors.");
                exit(0);
            } else {
                eprintln!(
                    "The file was parsed and error(s) were found: {}",
                    config_result.err().unwrap()
                );
                exit(1);
            }
        }
        SubCommand::ResourceUsage => {
            let cfg = config::read_config_file(&args.config)
                .ok()
                .unwrap_or_else(|| {
                    panic!("Failed to parse config file. Maybe run `verify` to check why?");
                });
            let mut resources = detect_resources(&args.resources_dir);
            for res in cfg.resources {
                let found = resources.remove(&res);
                match found {
                    Some(val) => println!("{} {} @ {}", "[  FOUND  ]".green(), res.bold(), val),
                    None => eprintln!("{} {}", "[ MISSING ]".red(), res.bold()),
                };
            }
            for key in resources.keys() {
                eprintln!(
                    "{} {} @ {}",
                    "[  EXTRA  ]".yellow(),
                    key.bold(),
                    &resources[key]
                );
            }
        }
        SubCommand::VersionServer(vs_args) => {
            let url;
            if vs_args.use_windows_server {
                url = "https://runtime.fivem.net/artifacts/fivem/build_server_windows/master/";
            } else {
                url = "https://runtime.fivem.net/artifacts/fivem/build_proot_linux/master/";
            }

            let mut art_serv = artifacts::ArtifactServer::new(url);

            if vs_args.get_url.is_some() {
                let for_version = vs_args.get_url.unwrap();
                let ar;
                if for_version.eq_ignore_ascii_case("latest") {
                    let latest_version = art_serv.get_latest_version_num();
                    ar = art_serv.get_artifact(latest_version);
                } else {
                    let for_version: u16 = for_version.parse().unwrap_or_else(|_| {
                        eprintln!("The version you specified is not valid!");
                        exit(1);
                    });
                    ar = art_serv.get_artifact(for_version);
                }

                if let Some(ar) = ar {
                    println!("{}\t{}", ar.num, ar.url);
                } else {
                    eprintln!("The artifact you requested doesn't exist!");
                }
            } else {
                let mut afs = art_serv.get_artifacts();
                afs.sort();
                for af in afs {
                    println!("{}\t{}", af.num, af.url);
                }
            }
        }
    };
}
