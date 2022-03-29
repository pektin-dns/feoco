use regex::Regex;
use std::fs::read_to_string;

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub headers: Headers,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Headers {
    #[serde(rename = "Content-Security-Policy")]
    pub content_security_policy: String,
    #[serde(rename = "x-frame-options")]
    pub x_frame_options: String,
    #[serde(rename = "x-content-type-options")]
    pub x_content_type_options: String,
    #[serde(rename = "x-permitted-cross-domain-policies")]
    pub x_permitted_cross_domain_policies: String,
    #[serde(rename = "x-download-options")]
    pub x_download_options: String,
    #[serde(rename = "x-xss-protection")]
    pub x_xss_protection: String,
    #[serde(rename = "referrer-policy")]
    pub referrer_policy: String,
    #[serde(rename = "Strict-Transport-Security")]
    pub strict_transport_security: String,
    #[serde(rename = "feature-policy")]
    pub feature_policy: String,
    #[serde(rename = "Cache-Control")]
    pub cache_control: String,
}
const CONFIG_PATH: &str = "/config.yml";

pub fn read_config() -> Config {
    let config_file = read_to_string(CONFIG_PATH).unwrap();
    let config_file = replace_variables(config_file);
    let config: Config = serde_yaml::from_str(&config_file).unwrap();
    config
}

pub fn replace_variables(config_file: String) -> String {
    let mut config_file = config_file;
    for matsch in Regex::new(r"\$\{[A-Z_]+\}")
        .unwrap()
        .find_iter(&config_file.clone())
    {
        let variable = matsch.as_str();
        let variable = variable.replace("${", "").replace('}', "");
        let variable_value = std::env::var(variable).unwrap_or_else(|_| "".to_string());
        config_file = config_file.replace(matsch.as_str(), &variable_value);
    }

    config_file
}
