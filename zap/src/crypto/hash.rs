use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

#[derive(Clone, Copy, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct HashValue {
    hash: [u8; HashValue::LENGTH],
}

impl HashValue {
    pub const LENGTH: usize = 32;

    pub fn new(hash: [u8; HashValue::LENGTH]) -> Self {
        HashValue { hash }
    }

    pub const fn zero() -> Self {
        HashValue {
            hash: [0; HashValue::LENGTH],
        }
    }
    
    pub fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, HashValueParseError> {
        let bytes = hex::decode(hex).map_err(|_| HashValueParseError)?;
        if bytes.len() != Self::LENGTH {
            return Err(HashValueParseError);
        }
        let mut hash = [0u8; Self::LENGTH];
        hash.copy_from_slice(&bytes);
        Ok(Self::new(hash))
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.hash)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HashValueParseError;

impl fmt::Display for HashValueParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unable to parse HashValue")
    }
}

impl std::error::Error for HashValueParseError {}

impl<'de> Deserialize<'de> for HashValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let s = String::deserialize(deserializer)?;
            let bytes = hex::decode(s).map_err(serde::de::Error::custom)?;
            if bytes.len() != Self::LENGTH {
                return Err(serde::de::Error::custom("Invalid hash length"));
            }
            let mut hash = [0u8; Self::LENGTH];
            hash.copy_from_slice(&bytes);
            Ok(HashValue::new(hash))
        } else {
             // In BCS, HashValue is serialized as a container with a single field.
             // We need to match the structure expected by Zapatos.
             // #[derive(Deserialize)]
             // #[serde(rename = "HashValue")]
             // struct Value { hash: [u8; 32] }
             
             // Actually, the `HashValue` implementation in `aptos-crypto` does explicit manual implementation.
             // When not human readable:
             // struct Value { hash: [u8; HashValue::LENGTH] }
             
            #[derive(Deserialize)]
            #[serde(rename = "HashValue")]
            struct Value {
                hash: [u8; HashValue::LENGTH],
            }
            let value = Value::deserialize(deserializer)?;
            Ok(HashValue::new(value.hash))
        }
    }
}

impl Serialize for HashValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&hex::encode(self.hash))
        } else {
            #[derive(Serialize)]
            #[serde(rename = "HashValue")]
            struct Value<'a> {
                hash: &'a [u8; HashValue::LENGTH],
            }
            Value { hash: &self.hash }.serialize(serializer)
        }
    }
}

impl fmt::Debug for HashValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.hash))
    }
}

impl fmt::Display for HashValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.hash))
    }
}

impl Default for HashValue {
    fn default() -> Self {
        Self::zero()
    }
}
