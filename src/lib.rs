#![allow(incomplete_features)]
#![feature(
    allocator_api,    // for Box::from_raw_in
    coerce_unsized,
    fn_traits,        // for forwarding `Fn` in `InplaceBox`
    generic_const_exprs,
    ptr_metadata,
    tuple_trait,      // for generalization of closure arguments
    unboxed_closures, // for forwarding `Fn` in `InplaceBox`
    unsize,
    specialization,   // for specialization of `ConvertIntoInplaceBox`;
                      // `min_specialization` does not apply for user defined trait.
)]
#![warn(clippy::all)]
#![cfg_attr(not(test), no_std)]
extern crate alloc;

mod impl_fn_traits;
mod impl_new;

use core::borrow::Borrow;
use core::borrow::BorrowMut;
use core::fmt;
use core::fmt::Debug;
use core::fmt::Display;
use core::fmt::Pointer;
use core::future::Future;
use core::marker::PhantomData;
use core::marker::Unsize;
use core::mem;
use core::mem::MaybeUninit;
use core::ops::Deref;
use core::ops::DerefMut;
use core::panic::AssertUnwindSafe;
use core::ptr;
use core::ptr::Pointee;

use mem::align_of;
use mem::size_of;

/// A container similar to `Box`, but without heap allocation.
///
/// It stores data inline within a fixed-size buffer.
pub struct InplaceBox<T: ?Sized, const SIZE: usize> {
    storage: MaybeUninit<[u8; SIZE]>,
    vtable: AssertUnwindSafe<<T as Pointee>::Metadata>,
    _phantom: PhantomData<T>,
}

impl<T: ?Sized, const SIZE: usize> InplaceBox<T, SIZE> {
    /// Helper to verify that the `T` is indeed a `dyn Trait`.
    const ASSERT_DYN_T: () = assert!(
        core::mem::size_of::<&T>() == core::mem::size_of::<usize>() * 2,
        "`InplaceBox` only works for `dyn Trait` types"
    );

    pub(crate) fn new_impl<'a, U: Sized + Unsize<T> + 'a>(value: U) -> Self {
        struct AssertSize<ValueT: Sized, DestT: Sized>(
            PhantomData<(ValueT, DestT)>,
        );
        impl<ValueT: Sized, DestT: Sized> AssertSize<ValueT, DestT> {
            const ASSERT: () = assert!(
                size_of::<ValueT>() <= size_of::<DestT>(),
                "Insufficient size of `InplaceBox` to store the object"
            );
            const fn check() {
                () = Self::ASSERT;
            }
        }
        AssertSize::<U, MaybeUninit<[u8; SIZE]>>::check();
        // SAFETY: Safe, since we just checked the size statically.
        unsafe { Self::new_unchecked(value) }
    }

    /// Construct a new object in-place in this object, without checking the
    /// size.
    ///
    /// The type of the value must be convertible to `dyn T`.
    ///
    /// This constructor is provided to allow constructing objects either in
    /// [`InplaceBox`] of a certain size or on heap for larger sizes. Since
    /// `if` conditions in the caller on the object size are not optimized
    /// out in debug mode, the code wouldn't compile when statically checking
    /// the size. With unchecked version, it's possible to have such
    /// dynamically-switched generics.
    ///
    /// # Safety
    ///
    /// The caller needs to ensure that the size of the type `U` is less than or
    /// equal to the `SIZE` of the `InplaceBox`. Only the type match and
    /// alignment is checked statically by the compiler.
    ///
    /// Prefer to use the safe [`Self::new`] constructor which checks for the
    /// size.
    pub unsafe fn new_unchecked<'a, U: Sized + Unsize<T> + 'a>(
        value: U,
    ) -> Self {
        struct AssertAlignment<ValueT: Sized, T: ?Sized>(
            PhantomData<(ValueT, T)>,
        );
        impl<ValueT: Sized, T: ?Sized> AssertAlignment<ValueT, T> {
            const ASSERT: () = assert!(
                align_of::<ValueT>() <= align_of::<<T as Pointee>::Metadata>(),
                "Value alignment exceeds maximum allowed alignment"
            );
            const fn check() {
                () = Self::ASSERT;
            }
        }
        AssertAlignment::<U, T>::check();
        () = Self::ASSERT_DYN_T;

        let value_ref: &T = &value;
        let vtable = AssertUnwindSafe(ptr::metadata(value_ref));
        let mut res = Self {
            storage: MaybeUninit::uninit(),
            vtable,
            _phantom: PhantomData,
        };
        unsafe { res.storage.as_mut_ptr().cast::<U>().write(value) };
        res
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

impl<T: ?Sized, const SIZE: usize> AsRef<T> for InplaceBox<T, SIZE> {
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T: ?Sized, const SIZE: usize> AsMut<T> for InplaceBox<T, SIZE> {
    fn as_mut(&mut self) -> &mut T {
        self
    }
}

impl<T: ?Sized, const SIZE: usize> Borrow<T> for InplaceBox<T, SIZE> {
    fn borrow(&self) -> &T {
        self
    }
}

impl<T: ?Sized, const SIZE: usize> BorrowMut<T> for InplaceBox<T, SIZE> {
    fn borrow_mut(&mut self) -> &mut T {
        self
    }
}

impl<T: ?Sized + Debug, const SIZE: usize> Debug for InplaceBox<T, SIZE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Debug::fmt(&**self, f)
    }
}

impl<T: ?Sized + Display, const SIZE: usize> Display for InplaceBox<T, SIZE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&**self, f)
    }
}

impl<T: ?Sized, const SIZE: usize> Pointer for InplaceBox<T, SIZE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ptr: *const T = &**self;
        Pointer::fmt(&ptr, f)
    }
}

impl<T: ?Sized + Future, const SIZE: usize> Future for InplaceBox<T, SIZE> {
    type Output = T::Output;

    fn poll(
        self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context<'_>,
    ) -> core::task::Poll<Self::Output> {
        // SAFETY: Safe, since we are just forwarding pinning to the inner
        // member, which is also pinned by definition.
        unsafe {
            let s = self.get_unchecked_mut();
            core::pin::Pin::new_unchecked(&mut **s).poll(cx)
        }
    }
}

impl<T: ?Sized, const SIZE: usize> Drop for InplaceBox<T, SIZE> {
    fn drop(&mut self) {
        unsafe {
            ptr::drop_in_place(self.as_mut_ptr());
        }
    }
}
