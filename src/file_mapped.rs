use {
    crate::{raw_place::RawPlace, utils, Error::CapacityOverflow, RawMem, Result},
    memmap2::{MmapMut, MmapOptions},
    std::{
        alloc::Layout,
        fmt::{self, Formatter},
        fs::File,
        io,
        mem::{self, MaybeUninit},
        path::Path,
        ptr::{self, NonNull},
    },
};

pub struct FileMapped<T> {
    buf: RawPlace<T>,
    mmap: Option<MmapMut>,
    pub(crate) file: File,
}

impl<T> FileMapped<T> {
    pub fn new(file: File) -> io::Result<Self> {
        const MIN_PAGE_SIZE: u64 = 4096;

        if file.metadata()?.len() < MIN_PAGE_SIZE {
            file.set_len(MIN_PAGE_SIZE)?;
        }

        Ok(Self { file, buf: RawPlace::dangling(), mmap: None })
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        File::options().create(true).read(true).write(true).open(path).and_then(Self::new)
    }

    fn map_yet(&mut self, cap: u64) -> io::Result<MmapMut> {
        unsafe { MmapOptions::new().len(cap as usize).map_mut(&self.file) }
    }

    unsafe fn assume_mapped(&mut self) -> &mut [u8] {
        self.mmap.as_mut().unwrap_unchecked()
    }
}

impl<T> RawMem for FileMapped<T> {
    type Item = T;

    fn allocated(&self) -> &[Self::Item] {
        unsafe { self.buf.as_slice() }
    }

    fn allocated_mut(&mut self) -> &mut [Self::Item] {
        unsafe { self.buf.as_slice_mut() }
    }

    unsafe fn grow(
        &mut self,
        addition: usize,
        fill: impl FnOnce(&mut [MaybeUninit<Self::Item>]),
    ) -> Result<&mut [Self::Item]> {
        let cap = self.buf.cap().checked_add(addition).ok_or(CapacityOverflow)?;
        // use layout to prevent all capacity bugs
        let layout = Layout::array::<T>(cap).map_err(|_| CapacityOverflow)?;
        let new_size = layout.size() as u64;

        // unmap the file by calling `Drop` of `mmap`
        let _ = self.mmap.take();

        if self.file.metadata()?.len() < new_size {
            self.file.set_len(new_size)?;
        }

        let ptr = unsafe {
            let mmap = self.map_yet(new_size)?;
            self.mmap.replace(mmap);
            // we set it now: ^^^
            NonNull::from(self.assume_mapped()) // it assume that `mmap` is some
        };

        Ok(self.buf.handle_fill(ptr.cast(), cap, fill))
    }

    fn shrink(&mut self, cap: usize) -> Result<()> {
        let cap = self.buf.cap().checked_sub(cap).expect("Tried to shrink to a larger capacity");
        self.buf.shrink_to(cap);

        let _ = self.mmap.take();

        let ptr = unsafe {
            // we can skip this checks because this memory layout is valid
            // then smaller layout will also be valid
            let new_size = mem::size_of::<T>().unchecked_mul(cap) as u64;
            self.file.set_len(new_size)?;

            let mmap = self.map_yet(new_size)?;
            self.mmap.replace(mmap);

            self.assume_mapped().into()
        };

        self.buf.set_ptr(ptr);

        Ok(())
    }
}

impl<T> Drop for FileMapped<T> {
    fn drop(&mut self) {
        unsafe {
            ptr::drop_in_place(self.buf.as_slice_mut());
        }

        let _ = self.file.sync_all();
    }
}

impl<T> fmt::Debug for FileMapped<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        utils::debug_mem(f, &self.buf, "FileMapped")?
            .field("mmap", &self.mmap)
            .field("file", &self.file)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use {super::*, std::io::Write};

    fn inner<M: RawMem>(mut mem: M, val: M::Item) -> Result<()>
    where
        M::Item: Clone,
    {
        mem.grow_filled(4, val)?;
        assert_eq!(mem.allocated().len(), 4);
        mem.shrink(4)?;
        assert_eq!(mem.allocated().len(), 0);
        Ok(())
    }

    #[test]
    fn test_inner() -> Result<()> {
        #[cfg(not(miri))]
        inner(FileMapped::new(tempfile::tempfile()?)?, "test".to_string())?;
        Ok(())
    }
}
