use core::marker::Unsize;

use crate::InplaceBox;

impl<T: ?Sized, const SIZE: usize> InplaceBox<T, SIZE> {
    /// Construct a new object in-place in this object.
    ///
    /// The type of the value must be convertible to `dyn T` and its size and
    /// alignment less than or equal to that of the `InplaceBox` space for
    /// the object.
    ///
    /// Type match, size and alignment are checked statically by the compiler.
    pub fn new<'a, U: Sized + Unsize<T> + 'a>(value: U) -> Self {
        Self::new_impl(value)
    }
}
