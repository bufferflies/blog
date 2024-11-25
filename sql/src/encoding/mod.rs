use std::collections::{BTreeSet, HashSet};

use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::error::Result;

pub mod bincode;
pub mod keycode;

/// Adds automatic Keycode encode/decode methods to key enums. These are
/// primarily meant for keys stored in key/value storage engines.
pub trait Key<'de>: Serialize + Deserialize<'de> {
    /// Decodes a key from a byte slice using Keycode.
    fn decode(bytes: &'de [u8]) -> Result<Self> {
        keycode::deserialize(bytes)
    }

    /// Encodes a key to a byte vector using Keycode.
    ///
    /// In the common case, the encoded key is borrowed for a storage engine
    /// call and then thrown away. We could avoid a bunch of allocations by
    /// taking a reusable byte vector to encode into and return a reference to
    /// it, but we keep it simple.
    fn encode(&self) -> Vec<u8> {
        keycode::serialize(self)
    }
}

/// Adds automatic Bincode encode/decode methods to value types. These are used
/// not only for values in key/value storage engines, but also for e.g. network
/// protocol messages and other values.
pub trait Value: Serialize + DeserializeOwned {
    /// Decodes a value from a byte slice using Bincode.
    fn decode(bytes: &[u8]) -> Result<Self> {
        bincode::deserialize(bytes)
    }

    /// Decodes a value from a reader using Bincode.
    fn decode_from<R: std::io::Read>(reader: R) -> Result<Self> {
        bincode::deserialize_from(reader)
    }

    /// Decodes a value from a reader using Bincode, or returns None if the
    /// reader is closed.
    fn maybe_decode_from<R: std::io::Read>(reader: R) -> Result<Option<Self>> {
        bincode::maybe_deserialize_from(reader)
    }

    /// Encodes a value to a byte vector using Bincode.
    fn encode(&self) -> Vec<u8> {
        bincode::serialize(self)
    }

    /// Encodes a value into a writer using Bincode.
    fn encode_into<W: std::io::Write>(&self, writer: W) -> Result<()> {
        bincode::serialize_into(writer, self)
    }
}

/// Blanket implementations for various types wrapping a value type.
impl<V: Value> Value for Option<V> {}
impl<V: Value> Value for Result<V> {}
impl<V: Value> Value for Vec<V> {}
impl<V1: Value, V2: Value> Value for (V1, V2) {}
impl<V: Value + std::cmp::Eq + std::hash::Hash> Value for HashSet<V> {}
impl<V: Value + std::cmp::Eq + std::cmp::Ord + std::hash::Hash> Value for BTreeSet<V> {}
