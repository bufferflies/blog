use std::{
    io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    path::PathBuf,
};

use fs4::FileExt;

use super::engine::Engine;
use crate::error::Result;

pub struct BitCask {
    log: Log,
    keydir: KeyDir,
}

// keys<-->(value pos and value lens)
// if value lens is None means the key is tombstone key
type KeyDir = std::collections::BTreeMap<Vec<u8>, (u64, u32)>;

impl BitCask {
    pub fn new(path: PathBuf) -> Result<Self> {
        log::info!("open database in {}", path.display());
        let mut log = Log::new(path.clone())?;
        let keydir = log.build_keydir()?;
        log::info!(
            "open database successful, path:{}, key size:{}",
            path.display(),
            keydir.len()
        );
        Ok(Self { log, keydir })
    }
}

impl Engine for BitCask {
    type ScanIterator<'a> = ScanIterator<'a>;

    fn delete(&mut self, key: &[u8]) -> Result<()> {
        self.log.write_entry(key, None)?;
        self.keydir.remove(key);
        Ok(())
    }

    fn flush(&mut self) -> Result<()> {
        #[cfg(not(test))]
        self.log.file.sync_all()?;
        Ok(())
    }

    fn get(&mut self, key: &[u8]) -> Result<Option<Vec<u8>>> {
        if let Some((value_pos, value_len)) = self.keydir.get(key) {
            Ok(Some(self.log.read_value(*value_pos, *value_len)?))
        } else {
            Ok(None)
        }
    }

    fn set(&mut self, key: &[u8], value: Vec<u8>) -> Result<()> {
        let (pos, len) = self.log.write_entry(key, Some(&*value))?;
        let value_len = value.len() as u32;
        self.keydir.insert(
            key.to_vec(),
            (pos + len as u64 - value_len as u64, value_len),
        );
        Ok(())
    }

    fn scan(&mut self, range: impl std::ops::RangeBounds<Vec<u8>>) -> Self::ScanIterator<'_> {
        ScanIterator { inner: self.keydir.range(range), log: &mut self.log }
    }
}

pub struct Log {
    path: PathBuf,
    file: std::fs::File,
}

impl Log {
    fn new(path: PathBuf) -> Result<Self> {
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?
        }
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)?;
        file.try_lock_exclusive()?;
        Ok(Self { path, file })
    }

    fn build_keydir(&mut self) -> Result<KeyDir> {
        let mut len_buf = [0u8; 4];
        let mut keydir = KeyDir::new();
        let file_len = self.file.metadata()?.len();
        let mut reader = BufReader::new(&mut self.file);
        let mut pos = reader.seek(SeekFrom::Start(0))?;
        while pos < file_len {
            let result = || -> std::result::Result<(Vec<u8>, u64, Option<u32>), std::io::Error> {
                reader.read_exact(&mut len_buf)?;
                let key_ken = u32::from_be_bytes(len_buf);
                reader.read_exact(&mut len_buf)?;
                let value_len_or_tombstone = match i32::from_be_bytes(len_buf) {
                    l if l >= 0 => Some(l as u32),
                    // -1 means tombstone
                    _ => None,
                };
                let value_pos = pos + 4 + 4 + key_ken as u64;

                let mut key_buffer = vec![0; key_ken as usize];
                reader.read_exact(&mut key_buffer)?;

                if let Some(value_len) = value_len_or_tombstone {
                    if value_pos + value_len as u64 > file_len {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::UnexpectedEof,
                            "value extends beyond end of file",
                        ));
                    }
                    reader.seek_relative(value_len as i64);
                }

                Ok((key_buffer, value_pos, value_len_or_tombstone))
            }();

            match result {
                Ok((key, value_pos, Some(value_len))) => {
                    keydir.insert(key, (value_pos, value_len));
                    pos = value_pos + value_len as u64;
                }
                Ok((key, value_pos, None)) => {
                    keydir.remove(&key);
                    pos = value_pos
                }
                Err(err) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
                    log::error!("Found incomplete entry at offset {}, truncating file", pos);
                    self.file.set_len(pos)?;
                    break;
                }
                Err(err) => return Err(err.into()),
            }
        }

        Ok(keydir)
    }

    fn read_value(&mut self, value_ops: u64, value_len: u32) -> Result<Vec<u8>> {
        let mut value = vec![0; value_len as usize];
        self.file.seek(SeekFrom::Start(value_ops))?;
        self.file.read_exact(&mut value);
        Ok(value)
    }

    fn write_entry(&mut self, key: &[u8], value: Option<&[u8]>) -> Result<(u64, u32)> {
        let key_len = key.len() as u32;
        let value_len = value.map_or(0, |v| v.len() as u32);
        // -1 means tombstone
        let value_len_or_tombstone = value.map_or(-1, |v| v.len() as i32);
        let len = 4 + 4 + key_len + value_len;
        let pos = self.file.seek(SeekFrom::End(0))?;
        let mut writer = BufWriter::with_capacity(len as usize, &mut self.file);
        writer.write_all(&key_len.to_be_bytes())?;
        writer.write_all(&value_len_or_tombstone.to_be_bytes())?;
        writer.write_all(key)?;
        if let Some(value) = value {
            writer.write_all(value)?;
        }
        writer.flush()?;
        Ok((pos, len))
    }
}


pub struct ScanIterator<'a> {
    inner: std::collections::btree_map::Range<'a, Vec<u8>, (u64, u32)>,
    log: &'a mut Log,
}

impl<'a> ScanIterator<'a> {
    fn map(&mut self, item: (&Vec<u8>, &(u64, u32))) -> <Self as Iterator>::Item {
        let (key, (value_pos, value_len)) = item;
        Ok((key.clone(), self.log.read_value(*value_pos, *value_len)?))
    }
}

impl<'a> Iterator for ScanIterator<'a> {
    type Item = Result<(Vec<u8>, Vec<u8>)>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|item| self.map(item))
    }
}

impl<'a> DoubleEndedIterator for ScanIterator<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back().map(|item| self.map(item))
    }
}