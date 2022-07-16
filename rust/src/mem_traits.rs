use std::io;

pub const DEFAULT_PAGE_SIZE: usize = 8 * 1024;

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

    fn grow_occupied(&mut self, capacity: usize) -> io::Result<&mut [T]> {
        let occupied = self.occupied();
        self.alloc(occupied + capacity)
    }

    fn shrink_occupied(&mut self, capacity: usize) -> io::Result<&mut [T]> {
        let occupied = self.occupied();
        self.alloc(occupied - capacity)
    }
}
