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

trait ConvertIntoInplaceBox<
    'a,
    T: ?Sized + 'a,
    U: Sized + Unsize<T> + 'a,
    const SIZE: usize,
>
{
    fn convert_into_inplace_box(self) -> InplaceBox<T, SIZE>;
}

impl<'a, T: ?Sized + 'a, U: Sized + Unsize<T> + 'a, const SIZE: usize>
    ConvertIntoInplaceBox<'a, T, U, SIZE> for U
{
    #[inline]
    default fn convert_into_inplace_box(self) -> InplaceBox<T, SIZE> {
        InplaceBox::new_impl(self)
    }
}

impl<
        'a,
        T: ?Sized + 'a,
        U: Sized + Unsize<T> + IsInPlaceBox<T, SIZE> + 'a,
        const SIZE: usize,
    > ConvertIntoInplaceBox<'a, T, U, SIZE> for U
{
    #[inline]
    fn convert_into_inplace_box(self) -> InplaceBox<T, SIZE> {
        self.move_out()
    }
}
