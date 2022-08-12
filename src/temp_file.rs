use crate::{base, FileMapped, RawMem, Result};
use std::{fs::File, io, path::Path};

/// Same as [`FileMapped`], but only allows temporary files
#[repr(transparent)]
pub struct TempFile<T>(FileMapped<T>);

impl<T> TempFile<T> {
    /// Constructs a new `TempFile` with temp file in [`std::env::temp_dir()`]
    /// with expansion handler.
    pub fn new_with(with: impl FnMut() -> T + 'static) -> io::Result<Self> {
        Self::from_file_with(tempfile::tempfile(), with)
    }

    /// Constructs a new `TempFile` with temp file in the specified directory
    /// with expansion handler.
    pub fn new_with_in<P: AsRef<Path>>(
        path: P,
        with: impl FnMut() -> T + 'static,
    ) -> io::Result<Self> {
        Self::from_file_with(tempfile::tempfile_in(path), with)
    }

    fn from_file_with(
        file: io::Result<File>,
        with: impl FnMut() -> T + 'static,
    ) -> io::Result<Self> {
        file.and_then(|file| FileMapped::new_with(file, with))
            .map(Self)
    }
}

impl<T: Default + 'static> TempFile<T> {
    /// Constructs a new `TempFile` with temp file in [`std::env::temp_dir()`]
    pub fn new() -> io::Result<Self> {
        Self::from_file(tempfile::tempfile())
    }

    /// Constructs a new `TempFile` with temp file in the specified directory.
    pub fn new_in<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        Self::from_file(tempfile::tempfile_in(path))
    }

    fn from_file(file: io::Result<File>) -> io::Result<Self> {
        Self::from_file_with(file, base::default_expand)
    }
}

impl<T> RawMem<T> for TempFile<T> {
    fn alloc(&mut self, capacity: usize) -> Result<&mut [T]> {
        self.0.alloc(capacity)
    }

    fn allocated(&self) -> usize {
        self.0.allocated()
    }

    // fixme: delegate all functions from `FileMapped`
}
