use crate::{FileMapped, RawMem, Result};
use std::{fs::File, io, path::Path};

#[repr(transparent)]
pub struct TempFile<T>(FileMapped<T>);

impl<T: Default> TempFile<T> {
    pub fn new() -> Result<Self> {
        Self::from_file(tempfile::tempfile())
    }

    pub fn new_in<P: AsRef<Path>>(path: P) -> Result<Self> {
        Self::from_file(tempfile::tempfile_in(path))
    }

    fn from_file(file: io::Result<File>) -> Result<Self> {
        file.map_err(Into::into)
            .and_then(|file| FileMapped::new(file).map(Self))
    }
}

impl<T: Default> RawMem<T> for TempFile<T> {
    fn alloc(&mut self, capacity: usize) -> Result<&mut [T]> {
        self.0.alloc(capacity)
    }

    fn allocated(&self) -> usize {
        self.0.allocated()
    }

    fn occupy(&mut self, capacity: usize) -> Result<()> {
        self.0.occupy(capacity)
    }

    fn occupied(&self) -> usize {
        self.0.occupied()
    }
}
