use crate::base::Base;
use crate::{internal, RawMem};
use memmap2::{MmapMut, MmapOptions};
use std::{
    cmp::max,
    error::Error,
    fs::File,
    io,
    mem::{size_of, ManuallyDrop},
    ptr,
    ptr::NonNull,
};

pub struct FileMappedMem<T> {
    base: Base<T>,
    allocated: usize,
    pub(crate) file: File,
    mapping: ManuallyDrop<MmapMut>, // TODO: `MaybeUninit`
}

impl<T: Default> FileMappedMem<T> {
    pub fn new(file: File) -> io::Result<Self> {
        let capacity = Base::<T>::MINIMUM_CAPACITY / size_of::<T>();
        let mapping = unsafe { MmapOptions::new().map_mut(&file)? };

        let len = file.metadata()?.len() as usize;
        let to_reserve = max(len, capacity);

        let mut new = Self {
            allocated: 0,
            base: Base::new(NonNull::slice_from_raw_parts(NonNull::dangling(), 0)),
            mapping: ManuallyDrop::new(mapping),
            file,
        };

        new.alloc_impl(to_reserve).map(|_| new)
    }

    unsafe fn map(&mut self, capacity: usize) -> io::Result<NonNull<[u8]>> {
        let mapping = MmapOptions::new().len(capacity).map_mut(&self.file)?;
        self.mapping = ManuallyDrop::new(mapping);
        Ok(NonNull::from(self.mapping.as_mut()))
    }

    unsafe fn unmap(&mut self) {
        ManuallyDrop::drop(&mut self.mapping)
    }

    fn alloc_impl(&mut self, capacity: usize) -> io::Result<()> {
        self.allocated = capacity;

        // SAFETY: `self.mapping` is initialized
        unsafe {
            self.unmap();
        }
        let file_len = self.file.metadata()?.len();
        self.file.set_len(file_len.max(capacity as u64))?;

        let ptr = unsafe { self.map(capacity) }?;
        let ptr =
            NonNull::slice_from_raw_parts(ptr.as_non_null_ptr(), self.allocated * size_of::<T>());
        self.base.ptr = internal::guaranteed_align_to(ptr);
        Ok(())
    }
}

impl<T: Default> RawMem<T> for FileMappedMem<T> {
    fn alloc(&mut self, capacity: usize) -> io::Result<&mut [T]> {
        self.alloc_impl(capacity)?;

        // SAFETY: `ptr` is valid slice
        unsafe { Ok(self.base.ptr.as_mut()) }
    }

    fn allocated(&self) -> usize {
        self.allocated
    }

    fn occupy(&mut self, capacity: usize) -> io::Result<()> {
        self.base.occupy(capacity)
    }

    fn occupied(&self) -> usize {
        self.base.occupied
    }
}

impl<T> Drop for FileMappedMem<T> {
    fn drop(&mut self) {
        // SAFETY: `slice` is valid file piece
        // SAFETY: items is friendly to drop
        unsafe {
            let ptr = self.base.ptr;
            let mut ptr = NonNull::slice_from_raw_parts(ptr.as_non_null_ptr(), self.allocated);
            let slice = ptr.as_mut();
            for item in slice {
                ptr::drop_in_place(item);
            }
        }

        // SAFETY: `self.mapping` is initialized
        unsafe {
            ManuallyDrop::drop(&mut self.mapping);
        }

        let _: Result<_, Box<dyn Error>> = try {
            self.file.set_len(self.allocated as u64)?;
            self.file.sync_all()?;
        };
    }
}
