use std::{
    borrow::Cow,
    collections::BTreeSet,
    sync::{Arc, Mutex, MutexGuard}, u64,
};

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use super::engine::Engine;
use crate::{
    encoding::{self, Key as _, Value as _,bincode},
    errdata, errinput,
    error::{Error, Result},
};

pub type Version = u64;

impl encoding::Value for Version {}

pub struct MVCC<E: Engine> {
    pub engine: Arc<Mutex<E>>,
}

impl<E: Engine> MVCC<E> {
    pub fn new(engine: E) -> Self {
        Self {
            engine: Arc::new(Mutex::new(engine)),
        }
    }

    pub fn begin(&self) -> Result<TransactionInner<E>> {
        TransactionInner::begin(self.engine.clone())
    }

    pub fn begin_read_only(&self) -> Result<TransactionInner<E>> {
        TransactionInner::begin_read_only(self.engine.clone(), None)
    }
}

pub struct TransactionInner<E: Engine> {
    pub engine: Arc<Mutex<E>>,
    pub st: TransactionState,
}

impl<E: Engine> TransactionInner<E> {
    fn begin(engine: Arc<Mutex<E>>) -> Result<Self> {
        let mut session = engine.lock()?;
        let version = match session.get(Key::NextVersion.encode().as_slice())? {
            Some(v) => Version::decode(v.as_slice())?,
            None => 0,
        };
        session.set(&Key::NextVersion.encode(), (version + 1).encode())?;
        let active = Self::scan_active(&mut session)?;
        if !active.is_empty() {
            session.set(&Key::TxnActiveSnapshot(version).encode(), active.encode())?;
        }
        session.set(&Key::TxnActive(version).encode(), vec![])?;
        drop(session);

        Ok(Self {
            engine,
            st: TransactionState {
                version,
                read_only: false,
                active,
            },
        })
    }

    pub fn begin_read_only(engine: Arc<Mutex<E>>, as_of: Option<Version>) -> Result<Self> {
        let mut session = engine.lock()?;
        let mut version = match session.get(Key::NextVersion.encode().as_slice())? {
            Some(v) => Version::decode(v.as_slice())?,
            None => 0,
        };
        let mut active = BTreeSet::new();
        if let Some(as_of) = as_of {
            if as_of >= version {
                return errinput!("version {as_of} does not exist");
            }
            version = as_of;
            if let Some(value) = session.get(&Key::TxnActiveSnapshot(version).encode())? {
                active = BTreeSet::<Version>::decode(&value)?;
            }
        } else {
            active = Self::scan_active(&mut session)?;
        }
        drop(session);
        Ok(Self {
            engine,
            st: TransactionState {
                version,
                read_only: true,
                active,
            },
        })
    }

    pub fn commit(self) -> Result<()> {
        if self.st.read_only {
            return Ok(());
        }
        let mut engine = self.engine.lock()?;
        let remove: Vec<_> = engine
            .scan_prefix(&KeyPrefix::TxnWrite(self.st.version).encode())
            .map_ok(|(k, _)| k)
            .try_collect()?;
        for key in remove {
            engine.delete(&key)?;
        }
        engine.delete(&Key::TxnActive(self.st.version).encode())
    }

    pub fn rollback(self) -> Result<()> {
        errinput!("rollback not implemented")
    }

    pub fn set(&self, key: &[u8], value: Vec<u8>) -> Result<()> {
        self.write_version(key, Some(value))
    }

    pub fn get(&self, _key: &[u8]) -> Result<Option<Vec<u8>>> {
        todo!()
    }

    fn scan_active(session: &mut MutexGuard<E>) -> Result<BTreeSet<Version>> {
        let mut active = BTreeSet::new();
        let mut scan = session.scan_prefix(&KeyPrefix::TxnActive.encode());
        while let Some((key, _)) = scan.next().transpose()? {
            match Key::decode(&key)? {
                Key::TxnActive(version) => active.insert(version),
                key => return errdata!("expect TxnActive, got {key:?}"),
            };
        }
        Ok(active)
    }

    fn write_version(&self, key: &[u8], value: Option<Vec<u8>>) -> Result<()> {
        if self.st.read_only {
            return Err(Error::ReadOnly);
        }
        let mut engine = self.engine.lock()?;
        let from = Key::Version(
            key.into(),
            self.st
                .active
                .iter()
                .min()
                .copied()
                .unwrap_or(self.st.version + 1),
        )
        .encode();
        let to=Key::Version(key.into(),u64::MAX).encode();
        if let Some((key,_))=engine.scan(from..=to).last().transpose()?{
            match Key::decode(&key)?{
                Key::Version(_,version)=>{
                    if !self.st.is_visible(version){
                        return Err(Error::Serialization)
                    }
                }
                key=>return errdata!("expect Version, got {key:?}")
            }
        }
        engine.set(&Key::TxnWrite(self.st.version,key.into()).encode(),vec![])?;
        engine.set(&Key::Version(key.into(), self.st.version).encode(),bincode::serialize(&value))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TransactionState {
    pub version: Version,
    pub read_only: bool,
    pub active: BTreeSet<Version>,
}

impl encoding::Value for TransactionState {}

impl TransactionState {
    pub fn is_visible(&self, version: Version) -> bool {
        if self.active.get(&version).is_some() {
            false
        } else if self.read_only {
            version < self.version
        } else {
            version <= self.version
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Key<'a> {
    NextVersion,
    /// uncommitted transactions by version
    TxnActive(Version),
    /// snapshot read by version.
    TxnActiveSnapshot(Version),

    TxnWrite(
        Version,
        #[serde(with = "serde_bytes")]
        #[serde(borrow)]
        Cow<'a, [u8]>,
    ),

    Version(
        #[serde(with = "serde_bytes")]
        #[serde(borrow)]
        Cow<'a, [u8]>,
        Version,
    ),
}

impl<'a> encoding::Key<'a> for Key<'a> {}

#[derive(Debug, Deserialize, Serialize)]
pub enum KeyPrefix<'a> {
    NextVersion,
    TxnActive,
    TxnActiveSnapshot,
    TxnWrite(Version),
    Version(
        #[serde(with = "serde_bytes")]
        #[serde(borrow)]
        Cow<'a, [u8]>,
    ),
}

impl<'a> encoding::Key<'a> for KeyPrefix<'a> {}
