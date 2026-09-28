//! A field the C# serializer skips.

/// A field C# marks `[SkipInState]`: derived or host state, in no state and no part of the machine's identity.
#[derive(Clone, Copy, Debug, Default)]
pub struct Skip<T>(pub T);

impl<T> PartialEq for Skip<T> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl<T> Eq for Skip<T> {}

impl<T> std::ops::Deref for Skip<T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> std::ops::DerefMut for Skip<T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}
