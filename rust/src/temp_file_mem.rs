use crate::{FileMappedMem, RawMem};

use std::io;

use std::ptr::NonNull;

#[repr(transparent)]
pub struct TempFileMem(FileMappedMem);

impl TempFileMem {
    pub fn new() -> io::Result<Self> {
        let file = tempfile::tempfile()?;
        Ok(TempFileMem(FileMappedMem::new(file)?))
    }
}

impl RawMem for TempFileMem {
    fn ptr(&self) -> NonNull<[u8]> {
        self.0.ptr()
    }

    fn alloc(&mut self, capacity: usize) -> io::Result<NonNull<[u8]>> {
        self.0.alloc(capacity)
    }

    fn allocated(&self) -> usize {
        self.0.allocated()
    }

    fn occupy(&mut self, capacity: usize) -> io::Result<NonNull<[u8]>> {
        self.0.occupy(capacity)
    }

    fn occupied(&self) -> usize {
        self.0.occupied()
    }
}
