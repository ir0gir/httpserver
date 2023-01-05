use std::collections::HashMap;
use std::env;
use std::fs::File;

use ansi_term::Color::Yellow;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_yaml::{self};

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    bind: String,
    #[serde(default)]
    alias: HashMap<String, String>,
    #[serde(default)]
    fallback_ports: Vec<u16>,
}

const CONFIG_FILE: &str = "config.yaml";

fn init_config() -> Config {
    let mut option_config: Option<Config> = None;

    let mut config_file = env::current_exe().unwrap();
    config_file.pop();
    config_file.push(CONFIG_FILE);

    match File::open(config_file)
    {
        Ok(file) => {
            let result = serde_yaml::from_reader(file);
            if result.is_ok() {
                option_config = Some(result.unwrap())
            }
        }
        Err(_) => {
            println!("{}", Yellow.paint("Could not open config file"));
        }
    }

    if option_config.is_some() {
        return option_config.unwrap();
    }
    Config { fallback_ports: vec![], alias: Default::default(), bind: "".to_string() }
}

pub fn fallback_ports() -> Vec<u16> {
    CONFIG.fallback_ports.clone()
}

pub fn bind() -> Vec<&'static str> {
    CONFIG.bind.split('|').collect()
}

lazy_static! {
    static ref CONFIG: Config = init_config();
}

pub fn resolve_aliases(asset: &String) -> &String {
    for (k, v) in &CONFIG.alias {
        if *asset == *k {
            return v;
        }
    }
    asset
}
