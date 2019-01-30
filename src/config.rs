extern crate colored;

use std::fs::File;
use std::io::Read;
use std::collections::HashMap;

use self::colored::*;

/// A struct containing *most* aspects of a FiveM server configuration file.
/// Some values have been excluded as there are few applications for including them, however
/// they can still be parsed manually.
#[derive(Debug)]
pub struct FivemConfig {
    pub hostname: String,
    pub resources: Vec<String>,
    pub convars: HashMap<String, String>,
    pub convars_replicated: HashMap<String, String>,
    pub allow_scripthook: bool,
    pub rcon_password: String,
    pub licensekey: String,
    pub server_icon: String,
    pub max_clients: u16,
}

impl FivemConfig {
    pub fn print_nicely(&self) {
        let mut hostname = String::new();
        let mut hostname_part = String::new();
        let mut is_escaped = false;
        let mut current_color = 0u8;
        for c in self.hostname.chars() {
            if c == '^' {
                hostname.push_str(&format!("{}", match current_color {
                    0 => hostname_part.white(),
                    1 => hostname_part.red(),
                    2 => hostname_part.green(),
                    3 => hostname_part.yellow(),
                    4 => hostname_part.blue(),
                    5 => hostname_part.bright_blue(),
                    6 => hostname_part.magenta(),
                    _ => hostname_part.white(),
                }));
                hostname_part = String::new();
                is_escaped = true;
            } else if is_escaped {
                current_color = c.to_string().parse::<u8>().unwrap_or_else(|_| {
                    panic!("A colour in the hostname is invalid!");
                });
                is_escaped = false;
            } else {
                hostname_part.push(c);
            }
        }
        hostname.push_str(&format!("{}", match current_color {
            0 => hostname_part.white(),
            1 => hostname_part.red(),
            2 => hostname_part.green(),
            3 => hostname_part.yellow(),
            4 => hostname_part.blue(),
            5 => hostname_part.bright_blue(),
            6 => hostname_part.magenta(),
            _ => hostname_part.white(),
        }));

        println!("{}: {}", "FiveM Server Configuration".underline(), hostname.italic());
        println!("  {}:   {}", "Script Hook".bold(), match self.allow_scripthook {
            true => "Allowed",
            false => "Disabled"
        });

        let mut rcon_formatted = String::new();
        if self.rcon_password.len() < 8 {
            for _ in 0..self.rcon_password.len() {
                rcon_formatted.push('*');
            }
        } else {
            for _ in 0..(self.rcon_password.len() - 4) {
                rcon_formatted.push('*');
            }
            rcon_formatted.push_str(&self.rcon_password[(self.rcon_password.len() - 4)..self.rcon_password.len()]);
        }
        println!("  {}: {}", "Rcon Password".bold(), rcon_formatted);

        let mut lkey_formatted = String::new();
        if self.licensekey.len() < 8 {
            for _ in 0..self.licensekey.len() {
                lkey_formatted.push('*');
            }
        } else {
            for _ in 0..(self.licensekey.len() - 4) {
                lkey_formatted.push('*');
            }
            lkey_formatted.push_str(&self.licensekey[(self.licensekey.len() - 4)..self.licensekey.len()]);
        }
        println!("  {}:   {}", "License Key".bold(), lkey_formatted);

        println!("  {}:   {}", "Server Icon".bold(), self.server_icon);
        println!("  {}:   {}", "Max Clients".bold(), self.max_clients);

        if self.convars.len() > 0 {
            println!("  {}:", "Convars".bold());
            let mut i = 0;
            let max = self.convars.keys().len();
            for key in self.convars.keys() {
                let val = &self.convars[key];
                i = i + 1;
                if max == i {
                    println!("   └─ {} = {}", key, val);
                } else {
                    println!("   ├─ {} = {}", key, val);
                }
            }
        }

        if self.convars_replicated.len() > 0 {
            println!("  {}:", "Replicated Convars".bold());
            let mut i = 0;
            let max = self.convars_replicated.keys().len();
            for key in self.convars_replicated.keys() {
                let val = &self.convars_replicated[key];
                i = i + 1;
                if max == i {
                    println!("   └─ {} = {}", key, val);
                } else {
                    println!("   ├─ {} = {}", key, val);
                }
            }
        }

        if self.resources.len() > 0 {
            println!("  {}:", "Resources".bold());
            let max = self.resources.len();
            for i in 0..max {
                let val = &self.resources[i];
                if max == (i + 1) {
                    println!("   └─ {}", val);
                } else {
                    println!("   ├─ {}", val);
                }
            }
        }
    }
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
    let lines: Vec<String> = file_contents
                                .replace("\r", "\n")   // Replace CR newlines with LF (extra blank lines don't matter)
                                .split("\n")           // Split on newlines
                                .map(|s: &str| s.to_string()).collect(); // Collect as Vec<String>

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
        } else if parts[0] == "setr" {
            config.convars_replicated.insert(parts[1].clone(), parts[2].clone());
        } else if parts[0] == "sv_scriptHookAllowed" {
            if parts[1] == "1" {
                config.allow_scripthook = true;
            } else {
                config.allow_scripthook = false;
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
        convars_replicated: HashMap::new(),
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
