#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(ptr_metadata)]
#![feature(unsize)]
#![feature(coerce_unsized)]

use std::marker::PhantomData;
use std::marker::Unsize;
use std::mem::MaybeUninit;
use std::mem::{self};
use std::ops::Deref;
use std::ops::DerefMut;
use std::panic::AssertUnwindSafe;
use std::ptr::Pointee;
use std::ptr::{self};

use mem::align_of;

/// A stack-allocated container similar to `Box`, but without heap allocation.
///
/// It stores data inline within a fixed-size buffer.
pub struct InplaceBox<T: ?Sized, const SIZE: usize = 0> {
    storage: MaybeUninit<[u8; SIZE]>,
    vtable: AssertUnwindSafe<<T as Pointee>::Metadata>,
    _phantom: PhantomData<T>,
}

impl<T: ?Sized, const SIZE: usize> InplaceBox<T, SIZE> {
    /// Creates a new `InplaceBox` containing the given value.
    ///
    /// # Panics
    ///
    /// Panics if:
    /// * The size of `U` exceeds `SIZE`
    /// * The alignment of `U` is greater than both the alignment of `[u8;
    ///   SIZE]` and the alignment of `<T as Pointee>::Metadata`
    pub fn new_with_size<U: Sized + Unsize<T>>(value: U) -> Self {
        assert!(
            size_of::<U>() <= SIZE,
            "Value size {} exceeds storage size {}",
            size_of::<U>(),
            SIZE
        );

        assert!(
            align_of::<U>() <= align_of::<[u8; SIZE]>()
                || align_of::<U>() <= align_of::<<T as Pointee>::Metadata>(),
            "Value alignment {} exceeds object_storage alignment: {} and exceeds vtable alignment: {}",
            align_of::<U>(),
            align_of::<[u8; SIZE]>(),
            align_of::<<T as Pointee>::Metadata>()
        );

        let mut storage = MaybeUninit::<[u8; SIZE]>::uninit();

        let ptr = storage.as_mut_ptr() as *mut U;

        unsafe {
            ptr::write(ptr, value);

            let metadata = ptr::metadata(ptr as *mut T);

            Self {
                storage,
                vtable: AssertUnwindSafe(metadata),
                _phantom: PhantomData,
            }
        }
    }

    /// Get a pointer to the contained value
    unsafe fn as_ptr(&self) -> *const T {
        let data_ptr = self.storage.as_ptr() as *const ();
        ptr::from_raw_parts(data_ptr, *self.vtable)
    }

    /// Get a mutable pointer to the contained value
    unsafe fn as_mut_ptr(&mut self) -> *mut T {
        let data_ptr = self.storage.as_mut_ptr() as *mut ();
        ptr::from_raw_parts_mut(data_ptr, *self.vtable)
    }
}

impl<T: ?Sized> InplaceBox<T, 0> {
    /// Create a new `InplaceBox` with size automatically calculated from the
    /// value.
    pub fn new<U: Sized + Unsize<T>>(
        value: U,
    ) -> InplaceBox<T, { size_of::<U>() }> {
        InplaceBox::new_with_size(value)
    }
}

impl<T: ?Sized, const SIZE: usize> Deref for InplaceBox<T, SIZE> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.as_ptr() }
    }
}

impl<T: ?Sized, const SIZE: usize> DerefMut for InplaceBox<T, SIZE> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.as_mut_ptr() }
    }
}

impl<T: ?Sized, const SIZE: usize> Drop for InplaceBox<T, SIZE> {
    fn drop(&mut self) {
        unsafe {
            ptr::drop_in_place(self.as_mut_ptr());
        }
    }
}
