use crate::crypto::HashValue;
use anyhow::{format_err, Error, Result};
use std::str::FromStr;
use std::fmt;

const WAYPOINT_DELIMITER: char = ':';

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Waypoint {
    version: u64,
    value: HashValue,
}

impl Waypoint {
    pub fn new_any(version: u64, value: HashValue) -> Self {
        Self {
            version,
            value,
        }
    }
    
    pub fn version(&self) -> u64 {
        self.version
    }
}

impl FromStr for Waypoint {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut split = s.split(WAYPOINT_DELIMITER);
        let version = split
            .next()
            .ok_or_else(|| format_err!("Failed to parse waypoint string {}", s))?
            .parse::<u64>()?;
        let value = HashValue::from_hex(
            split
                .next()
                .ok_or_else(|| format_err!("Failed to parse waypoint string {}", s))?,
        )?;
        Ok(Self { version, value })
    }
}

impl fmt::Display for Waypoint {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.version(),
            WAYPOINT_DELIMITER,
            self.value.to_hex()
        )
    }
}

