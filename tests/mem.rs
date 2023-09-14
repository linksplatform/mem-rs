use {
    platform_mem::RawMem,
    std::{error, result},
};

#[allow(dead_code)]
type Result = result::Result<(), Box<dyn error::Error>>;

pub fn grow_from_slice(mut mem: impl RawMem<Item = u8>) {
    assert_eq!(b"hello world", mem.grow_from_slice(b"hello world").unwrap());
}
