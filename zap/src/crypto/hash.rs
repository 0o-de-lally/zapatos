use hex::FromHex;
use rand::{distributions::Standard, prelude::Distribution, rngs::OsRng, Rng};
use serde::{de, ser, Deserialize, Serialize};
use std::{
    convert::{AsRef, TryFrom},
    fmt,
    str::FromStr,
};
use tiny_keccak::{Hasher, Sha3};

pub(crate) const HASH_PREFIX: &[u8] = b"APTOS::";

#[derive(Clone, Copy, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct HashValue {
    hash: [u8; HashValue::LENGTH],
}

impl HashValue {
    pub const LENGTH: usize = 32;
    pub const LENGTH_IN_BITS: usize = Self::LENGTH * 8;

    pub fn new(hash: [u8; HashValue::LENGTH]) -> Self {
        HashValue { hash }
    }

    pub fn from_slice<T: AsRef<[u8]>>(bytes: T) -> Result<Self, HashValueParseError> {
        <[u8; Self::LENGTH]>::try_from(bytes.as_ref())
            .map_err(|_| HashValueParseError)
            .map(Self::new)
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.hash.to_vec()
    }

    pub const fn zero() -> Self {
        HashValue {
            hash: [0; HashValue::LENGTH],
        }
    }

    pub fn random() -> Self {
        Self::random_with_rng(&mut OsRng)
    }

    pub fn random_with_rng<R: Rng>(rng: &mut R) -> Self {
        rng.r#gen()
    }

    pub fn sha3_256_of(buffer: &[u8]) -> Self {
        let mut sha3 = Sha3::v256();
        sha3.update(buffer);
        HashValue::from_keccak(sha3)
    }

    fn as_ref_mut(&mut self) -> &mut [u8] {
        &mut self.hash[..]
    }

    fn from_keccak(state: Sha3) -> Self {
        let mut hash = Self::zero();
        state.finalize(hash.as_ref_mut());
        hash
    }

    pub fn to_hex(&self) -> String {
        format!("{:x}", self)
    }

    pub fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, HashValueParseError> {
        <[u8; Self::LENGTH]>::from_hex(hex)
            .map_err(|_| HashValueParseError)
            .map(Self::new)
    }
}

impl ser::Serialize for HashValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        if serializer.is_human_readable() {
            serializer.serialize_str(&self.to_hex())
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

impl<'de> de::Deserialize<'de> for HashValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let encoded_hash = <String>::deserialize(deserializer)?;
            HashValue::from_hex(encoded_hash.as_str())
                .map_err(<D::Error as ::serde::de::Error>::custom)
        } else {
            #[derive(Deserialize)]
            #[serde(rename = "HashValue")]
            struct Value {
                hash: [u8; HashValue::LENGTH],
            }
            let value = Value::deserialize(deserializer)
                .map_err(<D::Error as ::serde::de::Error>::custom)?;
            Ok(Self::new(value.hash))
        }
    }
}

impl Default for HashValue {
    fn default() -> Self {
        HashValue::zero()
    }
}

impl AsRef<[u8; HashValue::LENGTH]> for HashValue {
    fn as_ref(&self) -> &[u8; HashValue::LENGTH] {
        &self.hash
    }
}

impl std::ops::Deref for HashValue {
    type Target = [u8; Self::LENGTH];
    fn deref(&self) -> &Self::Target {
        &self.hash
    }
}

impl fmt::LowerHex for HashValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if f.alternate() {
            write!(f, "0x")?;
        }
        for byte in &self.hash {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl fmt::Debug for HashValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HashValue(")?;
        <Self as fmt::LowerHex>::fmt(self, f)?;
        write!(f, ")")?;
        Ok(())
    }
}

impl fmt::Display for HashValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for byte in self.hash.iter().take(4) {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl FromStr for HashValue {
    type Err = HashValueParseError;
    fn from_str(s: &str) -> Result<Self, HashValueParseError> {
        HashValue::from_hex(s)
    }
}

impl Distribution<HashValue> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> HashValue {
        HashValue { hash: rng.r#gen() }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct HashValueParseError;
impl fmt::Display for HashValueParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unable to parse HashValue")
    }
}
impl std::error::Error for HashValueParseError {}

pub trait CryptoHasher: Default + std::io::Write {
    fn seed() -> &'static [u8; 32];
    fn update(&mut self, bytes: &[u8]);
    fn finish(self) -> HashValue;
}

#[derive(Clone)]
pub struct DefaultHasher {
    state: Sha3,
}

impl DefaultHasher {
    pub fn prefixed_hash(buffer: &[u8]) -> [u8; HashValue::LENGTH] {
        let salt: Vec<u8> = [HASH_PREFIX, buffer].concat();
        HashValue::sha3_256_of(&salt[..]).hash
    }

    pub fn new(typename: &[u8]) -> Self {
        let mut state = Sha3::v256();
        if !typename.is_empty() {
            state.update(&Self::prefixed_hash(typename));
        }
        DefaultHasher { state }
    }

    pub fn update(&mut self, bytes: &[u8]) {
        self.state.update(bytes);
    }

    pub fn finish(self) -> HashValue {
        let mut hasher = HashValue::default();
        self.state.finalize(hasher.as_ref_mut());
        hasher
    }
}
