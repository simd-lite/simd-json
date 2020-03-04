use crate::Value;

/// Functions guaranteed for any array object
pub trait Array {
    /// Elements of the array
    type Element: Value;

    /// Gets a ref to a value based on n index, returns `None` if the
    /// current Value isn't an Array or doesn't contain the index
    /// it was asked for.
    #[must_use]
    fn get(&self, i: usize) -> Option<&Self::Element>;

    /// Gets a ref to a value based on n index, returns `None` if the
    /// current Value isn't an Array or doesn't contain the index
    /// it was asked for.
    #[must_use]
    fn get_mut(&mut self, i: usize) -> Option<&mut Self::Element>;

    /// Returns the last element of the array or `None`
    #[must_use]
    fn pop(&mut self) -> Option<Self::Element>;

    /// Appends e to the end of the `Array`

    fn push(&mut self, e: Self::Element);
}

impl<T> Array for Vec<T>
where
    T: Value,
{
    type Element = T;
    #[inline]
    fn get(&self, i: usize) -> Option<&T> {
        <[T]>::get(self, i)
    }
    #[inline]
    fn get_mut(&mut self, i: usize) -> Option<&mut T> {
        <[T]>::get_mut(self, i)
    }

    #[inline]
    fn pop(&mut self) -> Option<T> {
        Vec::pop(self)
    }

    #[inline]
    fn push(&mut self, e: T) {
        Vec::push(self, e)
    }
}
