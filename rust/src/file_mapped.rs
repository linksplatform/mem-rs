use crate::{base::Base, internal, RawMem, Result};
use memmap2::{MmapMut, MmapOptions};
use std::{
    cmp::max,
    fs::File,
    mem::{size_of, ManuallyDrop},
    path::Path,
    ptr,
    ptr::NonNull,
};

pub struct FileMapped<T> {
    base: Base<T>,
    pub(crate) file: File,
    mapping: ManuallyDrop<MmapMut>, // TODO: `MaybeUninit`
}

impl<T: Default> FileMapped<T> {
    pub fn new(file: File) -> Result<Self> {
        let capacity = Base::<T>::MIN_CAPACITY / size_of::<T>();
        let mapping = unsafe { MmapOptions::new().map_mut(&file)? };

        let mut new = Self {
            base: Base::dangling(),
            mapping: ManuallyDrop::new(mapping),
            file,
        };

        new.alloc_impl(capacity).map(|_| new)
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        File::options()
            .create(true)
            .read(true)
            .write(true)
            .open(path)
            .map_err(Into::into)
            .and_then(Self::new)
    }

    unsafe fn map(&mut self, capacity: usize) -> Result<&mut [u8]> {
        let mapping = MmapOptions::new().len(capacity).map_mut(&self.file)?;
        self.mapping = ManuallyDrop::new(mapping);
        Ok(self.mapping.as_mut())
    }

    unsafe fn unmap(&mut self) {
        ManuallyDrop::drop(&mut self.mapping)
    }

    fn alloc_impl(&mut self, capacity: usize) -> Result<()> {
        let alloc_cap = capacity * size_of::<T>();

        // SAFETY: `self.mapping` is initialized
        unsafe {
            self.unmap();
        }
        let file_len = self.file.metadata()?.len();
        self.file.set_len(max(file_len, alloc_cap as u64))?;

        let bytes = unsafe { self.map(alloc_cap) }?;
        self.base.ptr = NonNull::from(internal::guaranteed_align_slice(bytes));
        Ok(())
    }
}

impl<T: Default> RawMem<T> for FileMapped<T> {
    fn alloc(&mut self, capacity: usize) -> Result<&mut [T]> {
        self.alloc_impl(capacity)?;

        // SAFETY: `ptr` is valid slice
        unsafe { Ok(self.base.ptr.as_mut()) }
    }

    fn allocated(&self) -> usize {
        self.base.allocated()
    }

    fn occupy(&mut self, capacity: usize) -> Result<()> {
        self.base.occupy(capacity)
    }

    fn occupied(&self) -> usize {
        self.base.occupied
    }
}

impl<T> Drop for FileMapped<T> {
    fn drop(&mut self) {
        // SAFETY: `slice` is valid file piece
        // SAFETY: items is friendly to drop
        unsafe {
            let ptr = self.base.ptr;
            let mut ptr =
                NonNull::slice_from_raw_parts(ptr.as_non_null_ptr(), self.base.allocated());
            let slice = ptr.as_mut();
            for item in slice {
                ptr::drop_in_place(item);
            }
        }

        // SAFETY: `self.mapping` is initialized
        unsafe {
            ManuallyDrop::drop(&mut self.mapping);
        }

        let _: Result<_> = try {
            self.file.sync_all()?;
        };
    }
}
