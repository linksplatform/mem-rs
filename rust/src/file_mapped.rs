use crate::{base::Base, internal, IsTrue, RawMem, Result};
use memmap2::{MmapMut, MmapOptions};
use std::{
    cmp::max,
    fs::File,
    io,
    mem::{size_of, ManuallyDrop},
    path::Path,
    ptr::{drop_in_place, NonNull},
};
use tap::Pipe;

pub struct FileMapped<T> {
    base: Base<T>,
    pub(crate) file: File,
    mapping: ManuallyDrop<MmapMut>, // TODO: `MaybeUninit`
}

impl<T: Default> FileMapped<T> {
    pub fn new(file: File) -> io::Result<Self> {
        let capacity = Base::<T>::MIN_CAPACITY / size_of::<T>();
        let mapping = unsafe { MmapOptions::new().map_mut(&file)? };

        Self {
            base: Base::dangling(),
            mapping: ManuallyDrop::new(mapping),
            file,
        }
        .pipe(|new| new.file.set_len(capacity as u64).map(|_| new))
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        File::options()
            .create(true)
            .read(true)
            .write(true)
            .open(path)
            .and_then(Self::new)
    }

    unsafe fn map(&mut self, capacity: usize) -> io::Result<&mut [u8]> {
        let mapping = MmapOptions::new().len(capacity).map_mut(&self.file)?;
        self.mapping = ManuallyDrop::new(mapping);
        Ok(self.mapping.as_mut())
    }

    unsafe fn unmap(&mut self) {
        ManuallyDrop::drop(&mut self.mapping)
    }
}

impl<T: Default> FileMapped<T> {
    fn alloc_impl(&mut self, capacity: usize) -> io::Result<()> {
        let alloc_cap = capacity * size_of::<T>();

        if capacity < self.base.allocated() {
            unsafe {
                self.base.handle_narrow(capacity);
            }
        }

        // SAFETY: `self.mapping` is initialized
        unsafe {
            self.unmap();
        }
        let file_len = self.file.metadata()?.len();
        self.file.set_len(max(file_len, alloc_cap as u64))?;

        let bytes = unsafe { self.map(alloc_cap) }?;

        // SAFETY: type is safe to slice from bytes
        unsafe {
            self.base.ptr = NonNull::from(internal::guaranteed_align_slice(bytes));
        }

        if capacity > self.base.allocated() {
            unsafe {
                self.base.handle_expand(capacity);
            }
        }

        Ok(())
    }
}

impl<T: Default> RawMem<T> for FileMapped<T>
where
    (): IsTrue<{ size_of::<T>() != 0 }>,
{
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
        // items is friendly to drop
        unsafe {
            let ptr = self.base.ptr;
            let mut ptr =
                NonNull::slice_from_raw_parts(ptr.as_non_null_ptr(), self.base.allocated());
            drop_in_place(ptr.as_mut());
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

unsafe impl<T: Sync> Sync for FileMapped<T> {}
unsafe impl<T: Send> Send for FileMapped<T> {}
