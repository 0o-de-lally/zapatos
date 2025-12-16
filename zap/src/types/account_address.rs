use anyhow::{bail, Result};
use hex::FromHex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt, str::FromStr};

#[derive(Clone, Copy, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct AccountAddress([u8; AccountAddress::LENGTH]);

impl AccountAddress {
    pub const LENGTH: usize = 32;

    pub const fn new(address: [u8; Self::LENGTH]) -> Self {
        Self(address)
    }

    pub const fn zero() -> Self {
        Self([0u8; Self::LENGTH])
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    pub fn to_hex(&self) -> String {
        format!("{:x}", self)
    }

    pub fn to_hex_literal(&self) -> String {
        format!("{:#x}", self)
    }

    pub fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self> {
        <[u8; Self::LENGTH]>::from_hex(hex)
            .map(Self)
            .map_err(|e| anyhow::anyhow!("Hex decode error: {}", e))
    }

    pub fn from_hex_literal(literal: &str) -> Result<Self> {
        let literal = literal.trim();
        if literal.starts_with("0x") {
            Self::from_hex(&literal[2..])
        } else {
            Self::from_hex(literal)
        }
    }
    
    pub fn from_bytes<T: AsRef<[u8]>>(bytes: T) -> Result<Self> {
        let bytes = bytes.as_ref();
        if bytes.len() != Self::LENGTH {
            bail!("Invalid length for AccountAddress");
        }
        let mut addr = [0u8; Self::LENGTH];
        addr.copy_from_slice(bytes);
        Ok(Self(addr))
    }
}

impl AsRef<[u8]> for AccountAddress {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl std::ops::Deref for AccountAddress {
    type Target = [u8; Self::LENGTH];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for AccountAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "0x")?;
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl fmt::Debug for AccountAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::LowerHex for AccountAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "0x")?;
        }
        for byte in &self.0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl FromStr for AccountAddress {
    type Err = anyhow::Error;

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        Self::from_hex_literal(str)
    }
}

impl Serialize for AccountAddress {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_hex_literal())
        } else {
            // BSC/binary serialization
            serializer.serialize_bytes(&self.0)
        }
    }
}

impl<'de> Deserialize<'de> for AccountAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = <String>::deserialize(deserializer)?;
            Self::from_hex_literal(&s).map_err(serde::de::Error::custom)
        } else {
             // We can use a tuple struct or just bytes, depending on how it's serialized.
             // APTOS default is `[u8; 32]`.
             let bytes = <[u8; 32]>::deserialize(deserializer)?;
             Ok(Self(bytes))
        }
    }
}
