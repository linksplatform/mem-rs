use crate::{FileMapped, RawMem, Result};



#[repr(transparent)]
pub struct TempFile<T>(FileMapped<T>);

impl<T: Default> TempFile<T> {
    pub fn new() -> Result<Self> {
        let file = tempfile::tempfile()?;
        Ok(TempFile(FileMapped::new(file)?))
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
