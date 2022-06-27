use crate::{FileMappedMem, RawMem};

use std::io;



#[repr(transparent)]
pub struct TempFileMem<T>(FileMappedMem<T>);

impl<T: Default> TempFileMem<T> {
    pub fn new() -> io::Result<Self> {
        let file = tempfile::tempfile()?;
        Ok(TempFileMem(FileMappedMem::new(file)?))
    }
}

impl<T: Default> RawMem<T> for TempFileMem<T> {
    fn alloc(&mut self, capacity: usize) -> io::Result<&mut [T]> {
        self.0.alloc(capacity)
    }

    fn allocated(&self) -> usize {
        self.0.allocated()
    }

    fn occupy(&mut self, capacity: usize) -> io::Result<()> {
        self.0.occupy(capacity)
    }

    fn occupied(&self) -> usize {
        self.0.occupied()
    }
}
