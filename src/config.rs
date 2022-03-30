use regex::Regex;
use std::{collections::HashMap, fs::read_to_string};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Config {
    pub headers: Headers,
}
#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct Headers {
    pub all: HashMap<String, String>,
    pub document: HashMap<String, String>,
}

const CONFIG_PATH: &str = "/config.yml";

pub fn read_config() -> Config {
    let config_file = read_to_string(CONFIG_PATH).unwrap();
    let config_file = replace_variables(config_file);
    let mut config: Config = serde_yaml::from_str(&config_file).unwrap();
    let replace_spaces = Regex::new(r"\s+").unwrap();
    for header in config.headers.all.clone() {
        config.headers.all.insert(
            header.0.to_string(),
            replace_spaces
                .replace_all(header.1.replace('\n', " ").as_str(), " ")
                .to_string(),
        );
    }
    for header in config.headers.document.clone() {
        config.headers.document.insert(
            header.0.to_string(),
            replace_spaces
                .replace_all(header.1.replace('\n', " ").as_str(), " ")
                .to_string(),
        );
    }
    config
}

pub fn replace_variables(config_file: String) -> String {
    let config_file = config_file;
    let mut config_file_repl = String::from("");
    for matsch in Regex::new(r"\$\{[A-Z_]+\}")
        .unwrap()
        .find_iter(&config_file)
    {
        let variable = matsch.as_str();
        let variable = variable.replace("${", "").replace('}', "");
        let variable_value = std::env::var(variable).unwrap_or_else(|_| "".to_string());
        config_file_repl = config_file.replace(matsch.as_str(), &variable_value);
    }

    config_file_repl
}
