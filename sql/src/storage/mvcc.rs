use std::{
    borrow::Cow,
    collections::{BTreeSet, VecDeque},
    ops::Bound,
    sync::{Arc, Mutex, MutexGuard},
    u64,
};

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use super::engine::{self, Engine};
use crate::{
    encoding::{self, Key as _, Value as _, bincode, keycode},
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
        if self.st.read_only {
            return Ok(());
        }
        let mut engine = self.engine.lock()?;
        let mut rollback = Vec::new();
        let mut scan = engine.scan_prefix(&KeyPrefix::TxnWrite(self.st.version).encode());
        while let Some((key, _)) = scan.next().transpose()? {
            match Key::decode(&key)? {
                Key::TxnWrite(_, key) => rollback.push(Key::Version(key, self.st.version).encode()),
                key => return errdata!("expect TxnWrite, got {key:?}"),
            }
        }
        drop(scan);
        for key in rollback.into_iter() {
            engine.delete(&key)?;
        }
        engine.delete(&Key::TxnActive(self.st.version).encode())
    }

    pub fn set(&self, key: &[u8], value: Vec<u8>) -> Result<()> {
        self.write_version(key, Some(value))
    }

    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let mut engine = self.engine.lock()?;
        let from = Key::Version(key.into(), 0).encode();
        let to = Key::Version(key.into(), self.st.version).encode();
        let mut scan = engine.scan(from..=to).rev();
        while let Some((key, value)) = scan.next().transpose()? {
            match Key::decode(&key)? {
                Key::Version(_, version) => {
                    if self.st.is_visible(version) {
                        return bincode::deserialize(&value);
                    }
                }
                key => return errdata!("expect Version, got {key:?}"),
            }
        }
        Ok(None)
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
        let to = Key::Version(key.into(), u64::MAX).encode();
        if let Some((key, _)) = engine.scan(from..=to).last().transpose()? {
            match Key::decode(&key)? {
                Key::Version(_, version) => {
                    if !self.st.is_visible(version) {
                        return Err(Error::Serialization);
                    }
                }
                key => return errdata!("expect Version, got {key:?}"),
            }
        }
        engine.set(&Key::TxnWrite(self.st.version, key.into()).encode(), vec![])?;
        engine.set(
            &Key::Version(key.into(), self.st.version).encode(),
            bincode::serialize(&value),
        )
    }

    pub fn scan_prefix(&self, prefix: &[u8]) -> ScanIterator<E> {
        let mut prefix = KeyPrefix::Version(prefix.into()).encode();
        prefix.truncate(prefix.len() - 2);
        let range = keycode::prefix_range(&prefix);
        ScanIterator::new(self.engine.clone(), self.st.clone(), range)
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

pub struct ScanIterator<E: Engine> {
    engine: Arc<Mutex<E>>,
    txn: TransactionState,
    buffer: VecDeque<(Vec<u8>, Vec<u8>)>,
    remainder: Option<(Bound<Vec<u8>>, Bound<Vec<u8>>)>,
}

impl<E: Engine> Clone for ScanIterator<E> {
    fn clone(&self) -> Self {
        Self {
            engine: self.engine.clone(),
            txn: self.txn.clone(),
            buffer: self.buffer.clone(),
            remainder: self.remainder.clone(),
        }
    }
}

impl<E: Engine> ScanIterator<E> {
    /// The number of live keys to pull from the engine at a time.
    #[cfg(not(test))]
    const BUFFER_SIZE: usize = 1000;
    /// Pull only 2 keys in tests, to exercise this more often.
    #[cfg(test)]
    const BUFFER_SIZE: usize = 2;

    fn new(
        engine: Arc<Mutex<E>>,
        txn: TransactionState,
        range: (Bound<Vec<u8>>, Bound<Vec<u8>>),
    ) -> Self {
        let buffer = VecDeque::with_capacity(Self::BUFFER_SIZE);
        Self {
            engine,
            txn,
            buffer,
            remainder: Some(range),
        }
    }

    fn fill_buffer(&mut self) -> Result<()> {
        if self.buffer.len() >= Self::BUFFER_SIZE {
            return Ok(());
        }
        let Some(range) = self.remainder.take() else {
            return Ok(());
        };
        let range_end = range.1.clone();
        let mut engine = self.engine.lock()?;
        let mut iter = VersionIterator::new(&self.txn, engine.scan(range)).peekable();
        while let Some((key, _, value)) = iter.next().transpose()? {
            match iter.peek() {
                Some(Ok((next, ..))) if next == &key => continue,
                Some(Err(err)) => return Err(err.clone()),
                Some(Ok(_)) | None => {}
            }
            let Some(value) = bincode::deserialize(&value)? else {
                continue;
            };
            self.buffer.push_back((key, value));
            if self.buffer.len() == Self::BUFFER_SIZE {
                if let Some((next, version, _)) = iter.next().transpose()? {
                    let range_start = Bound::Included(Key::Version(next.into(), version).encode());
                    self.remainder = Some((range_start, range_end));
                }
                return Ok(());
            }
        }
        Ok(())
    }
}

impl<E: Engine> Iterator for ScanIterator<E> {
    type Item = Result<(Vec<u8>, Vec<u8>)>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.buffer.is_empty() {
            if let Err(error) = self.fill_buffer() {
                return Some(Err(error));
            }
        }
        self.buffer.pop_front().map(Ok)
    }
}

struct VersionIterator<'a, I: engine::ScanIterator> {
    txn: &'a TransactionState,
    inner: I,
}

impl<'a, I: engine::ScanIterator> VersionIterator<'a, I> {
    fn new(txn: &'a TransactionState, inner: I) -> Self {
        Self { txn, inner }
    }

    fn try_next(&mut self) -> Result<Option<(Vec<u8>, Version, Vec<u8>)>> {
        while let Some((key, value)) = self.inner.next().transpose()? {
            let Key::Version(key, version) = Key::decode(&key)? else {
                return errdata!("expect Key::Version, got {key:?}");
            };
            if !self.txn.is_visible(version) {
                continue;
            }
            return Ok(Some((key.into_owned(), version, value)));
        }
        Ok(None)
    }
}

impl<'a, I: engine::ScanIterator> Iterator for VersionIterator<'a, I> {
    type Item = Result<(Vec<u8>, Version, Vec<u8>)>;
    fn next(&mut self) -> Option<Self::Item> {
        self.try_next().transpose()
    }
}
