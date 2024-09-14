use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Requirement {
    BREW,
    DOCKER,
    PYTHON,
    PIPX,
    POETRY,
}

fn default_registry() -> String {
    "docker.io".to_string()
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct ContainerSource {
    pub image: String,
    pub tag: String,
    pub start_command: Option<String>,
    pub stop_command: Option<String>,

    #[serde(default = "default_registry")]
    pub registry: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct AppSource {
    pub install_command: String,
    pub install_check_command: String,
    pub start_command: String,
    pub health_check_command: String,
    pub stop_command: Option<String>,
    pub clean_command: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
#[serde(untagged)]
pub enum Source {
    CONTAINER(ContainerSource),
    APP(AppSource),
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct PythonpathFeature {
    pub env_file_path: String,
    pub pythonpath_value: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(rename_all = "lowercase", tag = "name")]
pub enum Feature {
    PYTHONPATH(PythonpathFeature),
    BITWARDEN,
}

fn default_available_before_start() -> bool {
    true
}

fn default_exclude() -> Vec<String> {
    vec![]
}
fn default_rename() -> HashMap<String, String> {
    HashMap::new()
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct ExposedValueCommand {
    pub command: String,
    pub description: Option<String>,
    #[serde(default = "default_available_before_start")]
    pub available_before_start: bool,
    #[serde(default = "default_exclude")]
    pub exclude: Vec<String>,
    #[serde(default = "default_rename")]
    pub rename: HashMap<String, String>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct ExposedValueLiteral {
    pub name: String,
    pub value: String,
    pub description: Option<String>,
    #[serde(default = "default_available_before_start")]
    pub available_before_start: bool,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "lowercase", untagged)]
pub enum ExposedValueType {
    COMMAND(ExposedValueCommand),
    LITERAL(ExposedValueLiteral),
}

fn default_exposed_values() -> Vec<ExposedValueType> {
    vec![]
}

fn default_depends_on() -> Vec<String> {
    vec![]
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone)]
pub struct Service {
    pub name: String,

    #[serde(default = "default_exposed_values")]
    pub exposed_values: Vec<ExposedValueType>,

    pub source: Source,
    pub start_command: Option<String>,
    pub clean_command: Option<String>,

    #[serde(default = "default_depends_on")]
    pub depends_on: Vec<String>,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct RegistryCredentials {
    pub registry: String,
    pub username: String,
    pub password: String,
}

fn default_networks() -> Vec<String> {
    vec![]
}

fn default_requirements() -> Vec<Requirement> {
    vec![]
}

fn default_commands() -> HashMap<String, String> {
    HashMap::new()
}

fn default_pre_commands() -> HashMap<String, String> {
    HashMap::new()
}

fn default_services() -> Vec<Service> {
    vec![]
}

fn default_features() -> Vec<Feature> {
    vec![]
}

fn default_registry_credentials() -> Vec<RegistryCredentials> {
    vec![]
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Config {
    pub machine_name: String,

    pub machine_description: Option<String>,

    #[serde(default = "default_networks")]
    pub networks: Vec<String>,

    #[serde(default = "default_requirements")]
    pub requirements: Vec<Requirement>,

    #[serde(default = "default_commands")]
    pub commands: HashMap<String, String>,

    #[serde(default = "default_pre_commands")]
    pub pre_commands: HashMap<String, String>,

    #[serde(default = "default_services")]
    pub services: Vec<Service>,

    #[serde(default = "default_features")]
    pub features: Vec<Feature>,

    #[serde(default = "default_registry_credentials")]
    pub registry_credentials: Vec<RegistryCredentials>,
}
