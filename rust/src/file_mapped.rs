use crate::{base::Base, internal, IsTrue, RawMem, Result, DEFAULT_PAGE_SIZE};
use memmap2::{MmapMut, MmapOptions};
use std::{
    cmp::max,
    fs::File,
    io,
    mem::{size_of, ManuallyDrop},
    path::Path,
    ptr::drop_in_place,
};
use tap::Pipe;

/// [`RawMem`] that uses mapped file as space for a block of memory. It can change the file size
pub struct FileMapped<T> {
    base: Base<T>,
    pub(crate) file: File,
    mapping: ManuallyDrop<MmapMut>,
}

impl<T: Default> FileMapped<T> {
    /// Constructs a new `FileMapped` with provided file.
    /// File must be opened in read-write mode.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::{fs::File, io};
    /// use platform_mem::FileMapped;
    ///
    /// let file = File::options().read(true).write(true).open("file").unwrap();
    /// let mut mem: io::Result<FileMapped<usize>> = FileMapped::new(file);
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if file is not opened in read-write mode
    /// or it captured by other process.
    pub fn new(file: File) -> io::Result<Self> {
        let capacity = DEFAULT_PAGE_SIZE / size_of::<T>();
        let mapping = unsafe { MmapOptions::new().map_mut(&file)? };

        Self {
            base: Base::dangling(),
            mapping: ManuallyDrop::new(mapping),
            file,
        }
        .pipe(|new| new.file.set_len(capacity as u64).map(|_| new))
    }

    /// Constructs a new `FileMapped` with provided file,
    /// when open as read/write mode.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::io;
    /// use platform_mem::FileMapped;
    ///
    /// let mut mem: io::Result<FileMapped<usize>> = FileMapped::from_path("file");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if file is captured by other process.
    pub fn from_path<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        File::options()
            .create(true)
            .read(true)
            .write(true)
            .open(path)
            .and_then(Self::new)
    }

    unsafe fn map(&mut self, capacity: usize) -> io::Result<&mut [u8]> {
        self.mapping = MmapOptions::new()
            .len(capacity)
            .map_mut(&self.file)?
            .pipe(ManuallyDrop::new);
        self.mapping.as_mut().pipe(Ok)
    }

    unsafe fn unmap(&mut self) {
        ManuallyDrop::drop(&mut self.mapping)
    }
}

impl<T: Default> FileMapped<T> {
    fn alloc_impl(&mut self, capacity: usize) -> io::Result<()> {
        let cap = capacity * size_of::<T>();

        if capacity < self.base.allocated() {
            unsafe {
                self.base.handle_narrow(capacity);
            }
        }

        // SAFETY: `self.mapping` is initialized
        unsafe {
            self.unmap();
        }

        self.file
            .metadata()?
            .len()
            .pipe(|len| max(len, cap as u64))
            .pipe(|len| self.file.set_len(len))?;

        // SAFETY: type is safe to slice from bytes
        unsafe {
            self.base.ptr = self
                .map(cap)?
                .pipe(internal::safety_from_bytes_slice)
                .into()
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
        unsafe { self.base.ptr.as_mut().pipe(Ok) }
    }

    fn allocated(&self) -> usize {
        self.base.allocated()
    }
}

impl<T> Drop for FileMapped<T> {
    fn drop(&mut self) {
        // SAFETY: `slice` is valid file piece
        // `self.mapping` is initialized
        // items is friendly to drop
        unsafe {
            drop_in_place(self.mapping.as_mut());
            ManuallyDrop::drop(&mut self.mapping);
        }

        let _: Result<_> = try {
            self.file.sync_all()?;
        };
    }
}

unsafe impl<T: Sync> Sync for FileMapped<T> {}
unsafe impl<T: Send> Send for FileMapped<T> {}
