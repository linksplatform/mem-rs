use std::io;
use std::ptr::NonNull;

#[deprecated(note = "later use real compile time constant")]
pub const HOPE_PAGE_SIZE: usize = 8 * 1024;

pub trait RawMem /* Manager */ {
    fn ptr(&self) -> NonNull<[u8]>;

    fn alloc(&mut self, capacity: usize) -> io::Result<NonNull<[u8]>>;
    fn allocated(&self) -> usize;

    fn occupy(&mut self, capacity: usize) -> io::Result<NonNull<[u8]>>;
    fn occupied(&self) -> usize;
}
