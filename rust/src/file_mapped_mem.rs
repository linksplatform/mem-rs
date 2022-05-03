use crate::base::Base;
use crate::RawMem;
use memmap2::{MmapMut, MmapOptions};
use std::cmp::max;
use std::fs::File;
use std::io;
use std::mem::ManuallyDrop;
use std::ptr::NonNull;

pub struct FileMappedMem {
    base: Base,
    pub(in crate) file: File,
    mapping: ManuallyDrop<MmapMut>, // TODO: `MaybeUninit`
}

impl FileMappedMem {
    pub fn from_file(file: File) -> io::Result<Self> {
        let capacity = Base::MINIMUM_CAPACITY;
        let mapping = unsafe { MmapOptions::new().map_mut(&file)? };

        let len = file.metadata()?.len() as usize;
        let to_reserve = max(len, capacity);

        let mut new = Self {
            base: Base::new(NonNull::slice_from_raw_parts(NonNull::dangling(), 0)),
            mapping: ManuallyDrop::new(mapping),
            file,
        };

        new.alloc(to_reserve).map(|_| new)
    }

    pub fn new(file: File) -> std::io::Result<Self> {
        Self::from_file(file)
    }

    unsafe fn map(&mut self, capacity: usize) -> std::io::Result<NonNull<[u8]>> {
        let mapping = MmapOptions::new().len(capacity).map_mut(&self.file)?;
        self.mapping = ManuallyDrop::new(mapping);
        Ok(NonNull::from(self.mapping.as_mut()))
    }

    unsafe fn unmap(&mut self) {
        // TODO: WARNING! self.mapping must be initialized
        ManuallyDrop::drop(&mut self.mapping)
    }
}

impl RawMem for FileMappedMem {
    fn ptr(&self) -> NonNull<[u8]> {
        self.base.ptr()
    }

    fn alloc(&mut self, capacity: usize) -> io::Result<NonNull<[u8]>> {
        self.base.alloc(capacity)?;

        unsafe {
            self.unmap();
        }
        // TODO: file.set_len
        //  self.file.set_len(capacity as u64)?;

        // TODO: current impl
        let file_len = self.file.metadata()?.len();
        self.file.set_len(file_len.max(capacity as u64))?;

        let ptr = unsafe { self.map(capacity) }?;
        self.base.set_ptr(ptr);

        Ok(ptr)
    }

    fn allocated(&self) -> usize {
        self.base.allocated()
    }

    fn occupy(&mut self, capacity: usize) -> io::Result<NonNull<[u8]>> {
        self.base.occupy(capacity)
    }

    fn occupied(&self) -> usize {
        self.base.occupied()
    }
}

impl Drop for FileMappedMem {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.mapping);
        }
        // TODO: maybe remove `unwrap()` and ignore error
        self.file.set_len(self.allocated() as u64);
    }
}
