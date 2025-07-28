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

/// Helper to move the value w/o conversion for `InplaceBox`.
trait IsInPlaceBox<T: ?Sized, const SIZE: usize> {
    fn move_out(self) -> InplaceBox<T, SIZE>;
}

impl<T: ?Sized, const SIZE: usize> IsInPlaceBox<T, SIZE>
    for InplaceBox<T, SIZE>
{
    #[inline]
    fn move_out(self) -> InplaceBox<T, SIZE> {
        self
    }
}

/// Convert value into `InplaceBox`.
///
/// Provides specialized behavior: regular values are placed in-place,
/// existing `InplaceBox` values are moved without conversion.
trait ConvertIntoInplaceBox<'a, T, const SIZE: usize>
where T: ?Sized + 'a
{
    fn convert_into_inplace_box(self) -> InplaceBox<T, SIZE>;
}

impl<'a, T, U, const SIZE: usize> ConvertIntoInplaceBox<'a, T, SIZE> for U
where
    T: ?Sized + 'a,
    U: Sized + Unsize<T> + 'a,
{
    #[inline]
    default fn convert_into_inplace_box(self) -> InplaceBox<T, SIZE> {
        InplaceBox::new_impl(self)
    }
}

impl<'a, T, U, const SIZE: usize> ConvertIntoInplaceBox<'a, T, SIZE> for U
where
    T: ?Sized + 'a,
    U: Sized + Unsize<T> + IsInPlaceBox<T, SIZE> + 'a,
{
    #[inline]
    fn convert_into_inplace_box(self) -> InplaceBox<T, SIZE> {
        self.move_out()
    }
}
