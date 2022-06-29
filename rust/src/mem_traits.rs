use std::io;

pub const DEFAULT_PAGE_SIZE: usize = 8 * 1024;

pub trait RawMem<T> {
    fn alloc(&mut self, capacity: usize) -> io::Result<&mut [T]>;
    fn allocated(&self) -> usize;

    fn occupy(&mut self, capacity: usize) -> io::Result<()>;
    fn occupied(&self) -> usize;
}
