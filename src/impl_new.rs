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
        ConvertIntoInplaceBox::convert_into_inplace_box(value)
    }
}

trait ConvertIntoInplaceBox<'a, T: ?Sized + 'a, const SIZE: usize> {
    fn convert_into_inplace_box(self) -> InplaceBox<T, SIZE>;
}

impl<'a, T: ?Sized + 'a, U: Sized + Unsize<T> + 'a, const SIZE: usize>
    ConvertIntoInplaceBox<'a, T, SIZE> for U
{
    #[inline]
    default fn convert_into_inplace_box(self) -> InplaceBox<T, SIZE> {
        InplaceBox::new_impl(self)
    }
}

impl<'a, T: ?Sized + 'a, const SIZE: usize> ConvertIntoInplaceBox<'a, T, SIZE>
    for InplaceBox<T, SIZE>
{
    #[inline]
    fn convert_into_inplace_box(self) -> InplaceBox<T, SIZE> {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    trait Trait {}
    impl<T> Trait for T {}

    #[test]
    fn test_new() {
        let b1: InplaceBox<dyn Trait, 8> = InplaceBox::new(42u64);
        let _b2: InplaceBox<dyn Trait, 8> = InplaceBox::new(b1);
    }
}
