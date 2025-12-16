use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeMode {
    /// Streaming mode: in-memory state sync with debug logging
    Stream,
    /// Full node mode: persistent storage and full sync
    FullNode,
    /// Validator mode: consensus participation
    Validator,
}

impl fmt::Display for NodeMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeMode::Stream => write!(f, "stream"),
            NodeMode::FullNode => write!(f, "fullnode"),
            NodeMode::Validator => write!(f, "validator"),
        }
    }
}

impl FromStr for NodeMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "stream" => Ok(NodeMode::Stream),
            "fullnode" | "full" => Ok(NodeMode::FullNode),
            "validator" | "val" => Ok(NodeMode::Validator),
            _ => Err(format!("Invalid node mode: '{}'. Valid modes are: stream, fullnode, validator", s)),
        }
    }
}

impl Default for NodeMode {
    fn default() -> Self {
        NodeMode::Stream
    }
}
