//! Implementation of `Fn*` traits for `InplaceBox`.
//!
//! The `FnOnce` implementation uses a dummy allocator to temporarily create a
//! `Box` without actual memory allocation, allowing safe delegation to
//! `Box<F>`'s `FnOnce`.
//!
//! ## Why the intermediate `Box` is needed
//!
//! When `F` is a trait object like `dyn FnOnce<Args>`, we can't call
//! `F::call_once()` directly because `FnOnce::call_once` takes `self` by value,
//! requiring the compiler to know the size at compile time. But trait objects
//! are unsized types - their concrete size is only known at runtime.
//!
//! The `FnOnce` trait for trait objects is implemented for `Box<dyn
//! FnOnce<Args>>`, which handles the complex dynamic dispatch and destruction
//! logic. Rather than reimplementing this, we create a temporary `Box` with a
//! dummy allocator to reuse the existing, well-tested implementation.

use alloc::boxed::Box;
use core::alloc::AllocError;
use core::alloc::Allocator;
use core::alloc::Layout;
use core::marker::Tuple;
use core::ptr::NonNull;

use crate::InplaceBox;

impl<Args: Tuple, F: FnOnce<Args> + ?Sized, const SIZE: usize> FnOnce<Args>
    for InplaceBox<F, SIZE>
{
    type Output = <F as FnOnce<Args>>::Output;

    #[inline]
    extern "rust-call" fn call_once(mut self, args: Args) -> Self::Output {
        // SAFETY: Create a temporary `Box` with a dummy allocator that doesn't
        // actually deallocate. The original `InplaceBox` is forgotten to
        // prevent double-drop.
        let b = unsafe {
            Box::from_raw_in(self.as_mut(), InplaceBoxFnOnceDummyAllocator)
        };
        core::mem::forget(self); // the inner object is destroyed by `call_once` below
        <Box<_, _> as FnOnce<Args>>::call_once(b, args)
    }
}

/// Dummy allocator for `InplaceBox::call_once` that never allocates or
/// deallocates.
struct InplaceBoxFnOnceDummyAllocator;

// SAFETY: This allocator is unsafe. It is only to be used with `FnOnce` for
// `InplaceBox`.
unsafe impl Allocator for InplaceBoxFnOnceDummyAllocator {
    #[inline]
    fn allocate(&self, _layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        Err(AllocError) // in fact, never called
    }

    #[inline]
    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: Layout) {
        // No-op: memory owned by InplaceBox
    }
}

impl<Args: Tuple, F: FnMut<Args> + ?Sized, const SIZE: usize> FnMut<Args>
    for InplaceBox<F, SIZE>
{
    extern "rust-call" fn call_mut(&mut self, args: Args) -> Self::Output {
        <F as FnMut<Args>>::call_mut(self, args)
    }
}

impl<Args: Tuple, F: Fn<Args> + ?Sized, const SIZE: usize> Fn<Args>
    for InplaceBox<F, SIZE>
{
    extern "rust-call" fn call(&self, args: Args) -> Self::Output {
        <F as Fn<Args>>::call(self, args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fn_once() {
        let mut counter = 1;
        let adder: InplaceBox<dyn FnOnce<(usize,), Output = usize>, 32> =
            InplaceBox::new(|count| {
                let res = counter;
                counter = res + count;
                res
            });
        // call the function once
        let prev_value = adder(4);
        // previous count was 1
        assert_eq!(prev_value, 1);
        // we added 4, so now it's 5
        assert_eq!(5, counter);

        // let prev_value2 = adder(1); -- impossible - `FnOnce` call consumes
        // the box
    }

    #[test]
    fn fn_once_drop_or_call() {
        struct Guard<'a>(&'a mut bool);
        impl Drop for Guard<'_> {
            fn drop(&mut self) {
                *self.0 = true;
            }
        }

        // first part - ensure that the closure is dropped, if the `FnOnce` is
        // not called
        let mut called = false;
        let mut dropped = false;
        {
            let called = &mut called;
            let guard = Guard(&mut dropped);
            let b: InplaceBox<dyn FnOnce(), 32> = InplaceBox::new(move || {
                *called = true;
                core::mem::forget(guard);
            });

            drop(b); // drop w/o calling
        }
        assert!(!called);
        assert!(dropped);

        // second part - ensure that the closure is not dropped twice, the
        // `FnOnce` call via `InplaceBox` drops it
        called = false;
        dropped = false;
        {
            let called = &mut called;
            let guard = Guard(&mut dropped);
            let b: InplaceBox<dyn FnOnce<(), Output = ()>, 32> =
                InplaceBox::new(move || {
                    *called = true;
                    core::mem::forget(guard);
                });

            b(); // call it now
        }
        assert!(called);
        assert!(!dropped);
    }
}
