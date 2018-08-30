use std::fs::File;
use std::io::Read;
use std::collections::HashMap;

/// A struct containing *most* aspects of a FiveM server configuration file.
/// Some values have been excluded as there are few applications for including them, however
/// they can still be parsed manually.
#[derive(Debug)]
pub struct FivemConfig {
    hostname: String,
    resources: Vec<String>,
    convars: HashMap<String, String>,
    allow_scripthook: bool,
    rcon_password: String,
    licensekey: String,
    server_icon: String,
    max_clients: u16,
}

/// An internal function which takes a line from the config file and breaks it up into arguments,
/// generally by spaces but correctly dealing with quoting
fn config_line_split(line: String) -> Vec<String> {
    let mut parts = Vec::new();

    let mut in_text_block = false;
    let mut part = String::new();
    for c in line.chars() {
        if c == '"' {
            in_text_block = !in_text_block;
        } else if c == ' ' && !in_text_block {
            if part.len() != 0 {
                parts.push(part);
            }
            part = String::new();
        } else {
            part.push(c);
        }
    }

    if part.len() != 0 {
        parts.push(part);
    }

    parts
}

/// An internal function which takes a file and parses it, saving data into a `FivemConfig` struct.
fn parse_file(config: &mut FivemConfig, file_name: &str) -> Result<(), &'static str> {
    let mut file = File::open(file_name).ok().expect("Failed to open main config file.");
    let mut file_contents = String::new();

    file.read_to_string(&mut file_contents).ok().expect("Failed to read main config file.");
    let lines: Vec<String> = file_contents.split("\n").map(|s: &str| s.to_string()).collect();

    for line in lines {
        if line.starts_with("#") {
            continue;
        }

        let parts = config_line_split(line);
        if parts.len() == 0 {
            continue;
        }

        if parts[0] == "sv_hostname" {
            config.hostname = parts[1].clone();
        } else if parts[0] == "start" {
            config.resources.push(parts[1].clone());
        } else if parts[0] == "set" {
            config.convars.insert(parts[1].clone(), parts[2].clone());
        } else if parts[0] == "sv_scriptHookAllowed" {
            if parts[1] == "1" {
                config.allow_scripthook = true;
            }
        } else if parts[0] == "rcon_password" {
            config.rcon_password = parts[1].clone();
        } else if parts[0] == "sv_licenseKey" {
            config.licensekey = parts[1].clone();
        } else if parts[0] == "load_server_icon" {
            config.server_icon = parts[1].clone();
        } else if parts[0] == "sv_maxclients" {
            let parse_res = parts[1].parse::<u16>();
            if parse_res.is_ok() {
                config.max_clients = parse_res.ok().unwrap();
            } else {
                return Err("Max clients is not a number!");
            }
        } else if parts[0] == "exec" {
            let result = parse_file(config, &parts[1]);
            if result.is_err() {
                return Err(result.err().unwrap());
            }
        }
    }

    Ok(())
}

/// Reads a FiveM config file located by `file_name` and returns a `FivemConfig` struct containing
/// details about most of the configuration settings.
pub fn read_config_file<'a>(file_name: &'a str) -> Result<FivemConfig, &'static str> {
    let mut config = FivemConfig {
        hostname: String::new(),
        resources: Vec::new(),
        convars: HashMap::new(),
        allow_scripthook: true,
        rcon_password: String::new(),
        licensekey: String::new(),
        server_icon: String::new(),
        max_clients: 0,
    };

    let parse_result = parse_file(&mut config, file_name);

    if parse_result.is_err() {
        return Err(parse_result.err().unwrap());
    }

    Ok(config)
}
