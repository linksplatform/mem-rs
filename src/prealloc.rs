use {
    crate::{Error::CapacityOverflow, RawMem, Result},
    std::{
        mem::{self, MaybeUninit},
        ops::{Deref, DerefMut},
    },
};

/// Pre-allocated memory backed by an existing mutable slice or `Vec`.
///
/// This implementation does not perform dynamic allocation — it uses
/// a fixed buffer provided at construction time.
#[derive(Debug)]
pub struct PreAlloc<P> {
    place: P,
    used: usize,
}

impl<T, P: Deref<Target = [T]> + DerefMut> PreAlloc<P> {
    /// Constructs a new `PreAlloc` wrapping the given buffer.
    pub fn new(place: P) -> Self {
        Self { place, used: 0 }
    }
}

impl<T, P: Deref<Target = [T]> + DerefMut> RawMem for PreAlloc<P> {
    type Item = T;

    fn allocated(&self) -> &[Self::Item] {
        &self.place[..self.used]
    }

    fn allocated_mut(&mut self) -> &mut [Self::Item] {
        &mut self.place[..self.used]
    }

    unsafe fn grow(
        &mut self,
        addition: usize,
        fill: impl FnOnce(usize, (&mut [Self::Item], &mut [MaybeUninit<Self::Item>])),
    ) -> Result<&mut [Self::Item]> {
        let cap = self.used.checked_add(addition).ok_or(CapacityOverflow)?;
        let available = self.place.len();

        if let Some(slice) = self.place.get_mut(self.used..cap) {
            let uninit = unsafe { mem::transmute::<&mut [T], &mut [MaybeUninit<T>]>(&mut slice[..]) };
            fill(0, (&mut [], uninit));
            self.used = cap;
            Ok(slice)
        } else {
            Err(crate::Error::OverGrow { available, to_grow: cap })
        }
    }

    fn shrink(&mut self, cap: usize) -> Result<()> {
        self.used = self.used.checked_sub(cap).expect("Tried to shrink to a larger capacity");
        Ok(())
    }
}
