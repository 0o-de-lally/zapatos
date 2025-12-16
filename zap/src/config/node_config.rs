use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct NodeConfig {
    #[serde(default)]
    pub base: BaseConfig,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct BaseConfig {
    pub data_dir: PathBuf,
    pub role: RoleType,
    pub waypoints: Option<String>, // simplified for now
}

impl Default for BaseConfig {
    fn default() -> Self {
        BaseConfig {
            data_dir: PathBuf::from("/opt/aptos/data"),
            role: RoleType::FullNode,
            waypoints: None,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RoleType {
    Validator,
    FullNode,
}

impl Default for RoleType {
    fn default() -> Self {
        RoleType::FullNode
    }
}

impl NodeConfig {
    pub fn load_from_path<P: AsRef<Path>>(input_path: P) -> anyhow::Result<Self> {
        let contents = std::fs::read_to_string(input_path)?;
        let config = serde_yaml::from_str(&contents)?;
        Ok(config)
    }
}
