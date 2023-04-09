use {
    crate::{Alloc, RawMem, Result},
    std::{
        alloc::{self},
        mem::MaybeUninit,
    },
};

pub struct Global<T>(Alloc<T, alloc::Global>);

impl<T> Global<T> {
    pub const fn new() -> Self {
        Self(Alloc::new(alloc::Global))
    }
}

impl<T> RawMem for Global<T> {
    type Item = T;

    fn allocated(&self) -> &[Self::Item] {
        self.0.allocated()
    }

    fn allocated_mut(&mut self) -> &mut [Self::Item] {
        self.0.allocated_mut()
    }

    unsafe fn grow(
        &mut self,
        cap: usize,
        fill: impl FnOnce(&mut [MaybeUninit<Self::Item>]),
    ) -> Result<&mut [Self::Item]> {
        self.0.grow(cap, fill)
    }

    fn shrink(&mut self, cap: usize) -> Result<()> {
        self.0.shrink(cap)
    }
}

#[test]
fn for_global_test() -> Result<()> {
    crate::tests::inner(Global::new(), "lol".to_string())?;
    Ok(())
}
