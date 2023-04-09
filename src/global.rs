use {
    crate::{Alloc, RawMem, RawPlace, Result},
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

#[cfg(test)]
mod tests {
    use super::*;
    fn inner<M: RawMem>(mut mem: M, val: M::Item) -> Result<()>
    where
        M::Item: Clone,
    {
        mem.grow_filled(10, val)?;
        assert_eq!(mem.allocated().len(), 10);
        mem.shrink(10)?;
        assert_eq!(mem.allocated().len(), 0);
        Ok(())
    }

    #[test]
    fn for_globalg_test() -> Result<()> {
        inner(Global::new(), "lol".to_string())?;
        Ok(())
    }
}
