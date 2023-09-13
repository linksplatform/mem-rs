use {platform_mem::RawMem, std::error::Error};

type Result = std::result::Result<(), Box<dyn Error>>;

pub fn mem(mut mem: impl RawMem<Item = String>) -> Result {
    assert_eq!(&["hello world".to_string()], mem.grow_from_slice(&["hello world".to_string()])?);

    Ok(())
}
