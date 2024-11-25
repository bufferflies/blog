use crate::{encoding::keycode, error::Result};

pub trait ScanIterator: DoubleEndedIterator<Item = Result<(Vec<u8>, Vec<u8>)>> {}
pub trait Engine: Send {
    type ScanIterator<'a>: ScanIterator + 'a
    where
        Self: Sized + 'a;

    fn delete(&mut self, key: &[u8]) -> Result<()>;

    fn get(&mut self, key: &[u8]) -> Result<Option<Vec<u8>>>;

    fn set(&mut self, key: &[u8], value: Vec<u8>) -> Result<()>;

    fn flush(&mut self) -> Result<()>;

    fn scan(&mut self, range: impl std::ops::RangeBounds<Vec<u8>>) -> Self::ScanIterator<'_>
    where
        Self: Sized;

    fn scan_prefix(&mut self, prefix: &[u8]) -> Self::ScanIterator<'_>
    where
        Self: Sized,
    {
        self.scan(keycode::prefix_range(prefix))
    }
}
