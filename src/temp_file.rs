use {
    crate::{FileMapped, RawMem, Result},
    std::mem::MaybeUninit,
};

pub struct TempFile<T>(FileMapped<T>);

impl<T> TempFile<T> {
    pub fn new() -> Result<Self> {
        Ok(Self(FileMapped::new(tempfile::tempfile()?)?))
    }
}

impl<T> RawMem for TempFile<T> {
    type Item = T;

    fn allocated(&self) -> &[Self::Item] {
        self.0.allocated()
    }

    fn allocated_mut(&mut self) -> &mut [Self::Item] {
        self.0.allocated_mut()
    }

    unsafe fn grow(
        &mut self,
        addition: usize,
        fill: impl FnOnce(&mut [MaybeUninit<Self::Item>]),
    ) -> Result<&mut [Self::Item]> {
        self.0.grow(addition, fill)
    }

    fn shrink(&mut self, cap: usize) -> Result<()> {
        self.0.shrink(cap)
    }
}

#[test]
fn for_temp_file_teпst() -> Result<()> {
    crate::tests::inner(TempFile::new()?, "test".to_string())?;
    Ok(())
}
