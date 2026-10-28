use {
    crate::{Error::CapacityOverflow, RawMem, Result, raw_place::RawPlace, utils},
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

/// Memory-mapped file storage.
///
/// Elements are stored directly in a memory-mapped region backed by a file.
/// The file is resized and remapped on each [`grow`](RawMem::grow) or
/// [`shrink`](RawMem::shrink) call. Data is synced to disk on drop.
///
/// The file is guaranteed to be at least 4 KiB (one page) to satisfy
/// OS-level mmap requirements.
///
/// Growth through [`grow_filled`](RawMem::grow_filled),
/// [`grow_zeroed`](RawMem::grow_zeroed), or [`grow_with`](RawMem::grow_with)
/// preserves complete elements present when the file was opened and initializes
/// only the new tail. Padding added on opening is initialized on first growth.
pub struct FileMapped<T> {
    buf: RawPlace<T>,
    mmap: Option<MmapMut>,
    pub(crate) file: File,
    // Existing file data plus successfully initialized growth, excluding fresh
    // minimum-page padding. This can exceed the currently mapped region.
    initialized_size: u64,
}

impl<T> FileMapped<T> {
    /// Creates a `FileMapped` from an already-opened file.
    ///
    /// If the file is smaller than one page (4 KiB), it is extended.
    pub fn new(file: File) -> io::Result<Self> {
        const MIN_PAGE_SIZE: u64 = 4096;

        let initialized_size = file.metadata()?.len();
        if initialized_size < MIN_PAGE_SIZE {
            file.set_len(MIN_PAGE_SIZE)?;
        }

        Ok(Self { file, buf: RawPlace::dangling(), mmap: None, initialized_size })
    }

    /// Opens (or creates) a file at `path` and wraps it in a `FileMapped`.
    pub fn from_path<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        File::options()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(path)
            .and_then(Self::new)
    }

    fn map_yet(&mut self, cap: u64) -> io::Result<MmapMut> {
        unsafe { MmapOptions::new().len(cap as usize).map_mut(&self.file) }
    }

    unsafe fn assume_mapped(&mut self) -> &mut [u8] {
        unsafe { self.mmap.as_mut().unwrap_unchecked() }
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
        fill: impl FnOnce(usize, (&mut [T], &mut [MaybeUninit<T>])),
    ) -> Result<&mut [T]> {
        unsafe {
            let cap = self.buf.cap().checked_add(addition).ok_or(CapacityOverflow)?;
            // use layout to prevent all capacity bugs
            let layout = Layout::array::<T>(cap).map_err(|_| CapacityOverflow)?;
            let new_size = layout.size() as u64;

            // unmap the file by calling `Drop` of `mmap`
            let _ = self.mmap.take();

            let old_size = self.file.metadata()?.len();

            if old_size < new_size {
                self.file.set_len(new_size)?;
            }
            let inited = (self.initialized_size / mem::size_of::<T>() as u64)
                .saturating_sub(self.buf.cap() as u64)
                .min(addition as u64) as usize;

            let ptr = {
                let mmap = self.map_yet(new_size)?;
                self.mmap.replace(mmap);
                // we set it now: ^^^
                NonNull::from(self.assume_mapped()) // it assume that `mmap` is some
            };

            let grown = self.buf.handle_fill((ptr.cast(), cap), inited, fill);
            self.initialized_size = self.initialized_size.max(new_size);
            Ok(grown)
        }
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
            self.initialized_size = new_size;

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
