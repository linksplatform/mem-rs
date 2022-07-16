use std::alloc::Global;
use std::io;

// Bare metal platforms usually have very small amounts of RAM
// (in the order of hundreds of KB)
#[rustfmt::skip]
pub const DEFAULT_PAGE_SIZE: usize = if cfg!(target_os = "espidf") { 512 } else { 8 * 1024 };

pub trait RawMem<T> {
    fn alloc(&mut self, capacity: usize) -> io::Result<&mut [T]>;
    fn allocated(&self) -> usize;

    fn occupy(&mut self, capacity: usize) -> io::Result<()>;
    fn occupied(&self) -> usize;

    fn grow(&mut self, capacity: usize) -> io::Result<&mut [T]> {
        let allocated = self.allocated();
        self.alloc(allocated + capacity)
    }

    fn shrink(&mut self, capacity: usize) -> io::Result<&mut [T]> {
        let allocated = self.allocated();
        self.alloc(allocated - capacity)
    }

    fn grow_occupied(&mut self, capacity: usize) -> io::Result<()> {
        let occupied = self.occupied();
        self.occupy(occupied + capacity)
    }

    fn shrink_occupied(&mut self, capacity: usize) -> io::Result<()> {
        let occupied = self.occupied();
        self.occupy(occupied - capacity)
    }
}
