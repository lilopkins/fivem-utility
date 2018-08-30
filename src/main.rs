extern crate clap;

use clap::{Arg, App, SubCommand};

use std::process::exit;

pub mod config;

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
                    .subcommand(SubCommand::with_name("git-update-check")
                            .about("Checks addons which are git repositories for updates."))
                    .get_matches();

    let config_file = matches.value_of("config").unwrap_or("server.cfg");
    let resources_dir = matches.value_of("resources-dir").unwrap_or("resources");

    if let Some(_) = matches.subcommand_matches("print") {
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

    } else if let Some(matches) = matches.subcommand_matches("git-update-check") {
        unimplemented!();
    } else {
        eprintln!("You must specify a subcommand. See --help for more information.");
        exit(1);
    }
}
