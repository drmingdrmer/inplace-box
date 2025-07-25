// Need to add generic_const_exprs feature in the test too.
// See: https://github.com/rust-lang/rust/issues/133199#issuecomment-2630615573
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use inplace_box::InplaceBox;

#[test]
fn test_method() {
    trait MyTrait {
        fn method(&self) -> i32;
    }

    static_assertions::assert_eq_size!(InplaceBox<dyn MyTrait, 8>, [usize; 2]);
    static_assertions::assert_eq_size!(
        Option<InplaceBox<dyn MyTrait, 8>>,
        InplaceBox<dyn MyTrait, 8>
    );

    struct MyStruct(i32, Arc<()>);

    impl MyTrait for MyStruct {
        fn method(&self) -> i32 {
            let _ = self.1.clone();
            self.0
        }
    }

    let b = InplaceBox::<dyn MyTrait, 16>::new(MyStruct(42, Arc::new(())));
    assert_eq!(b.method(), 42);
}

#[test]
fn test_drop() {
    trait MyTrait {}

    struct MyStruct {
        drop_count: Arc<AtomicU64>,
    }

    impl MyTrait for MyStruct {}

    let drop_count = Arc::new(AtomicU64::new(0));

    impl Drop for MyStruct {
        fn drop(&mut self) {
            self.drop_count.fetch_add(1, Ordering::Relaxed);
        }
    }

    let v = MyStruct {
        drop_count: drop_count.clone(),
    };
    assert_eq!(0, drop_count.load(Ordering::Relaxed));

    {
        let _f = InplaceBox::<dyn MyTrait, 8>::new(v);
    }
    assert_eq!(1, drop_count.load(Ordering::Relaxed), "drop is called");

    let v = MyStruct {
        drop_count: drop_count.clone(),
    };
    {
        let _f = InplaceBox::<dyn MyTrait, 8>::new(v);
    }
    assert_eq!(2, drop_count.load(Ordering::Relaxed), "drop is called");
}

/// Put various types into one vec
#[test]
fn test_erase_type() {
    trait MyTrait {
        fn method(&self) -> i32;
    }

    struct MyStruct1(i32);
    impl MyTrait for MyStruct1 {
        fn method(&self) -> i32 {
            self.0
        }
    }

    struct MyStruct2(i32);
    impl MyTrait for MyStruct2 {
        fn method(&self) -> i32 {
            self.0 * 2
        }
    }

    let arr: Vec<InplaceBox<dyn MyTrait, 4>> =
        vec![InplaceBox::new(MyStruct1(1)), InplaceBox::new(MyStruct2(2))];

    let result = arr.into_iter().map(|b| b.method()).collect::<Vec<_>>();

    assert_eq!(result, vec![1, 4]);
}

#[test]
#[cfg(not(miri))] // `miri` reports the error, since we intentionally cause an UB
fn unchecked() {
    trait MyTrait {
        fn method(&self) -> i32;
    }

    impl MyTrait for i32 {
        fn method(&self) -> i32 {
            *self
        }
    }

    // SAFETY: This is intentionally unsafe and broken, but we do have
    // sufficient space to materialize the 4B integer there (due to
    // 8B alignment).
    let b: InplaceBox<dyn MyTrait, 3> =
        unsafe { InplaceBox::new_unchecked(4711_i32) };
    assert_eq!(4711, b.method() & 0xff_ffff);
}

#[doc(hidden)]
mod compile_tests {
    #![allow(dead_code)]

    /// The test ensures that the box must be sufficiently large:
    /// ```compile_fail
    /// use inplace_box::*;
    /// trait Trait {}
    /// impl Trait for i32 {}
    /// let _b: InplaceBox<dyn Trait, 3> = InplaceBox::new(4711);
    /// ```
    fn fail_for_too_small_size() {}

    /// Verification for `fail_for_too_small_size()` to check that it doesn't
    /// fail with sufficient size:
    /// ```
    /// use inplace_box::*;
    /// trait Trait {}
    /// impl Trait for i32 {}
    /// let _b: InplaceBox<dyn Trait, 4> = InplaceBox::new(4711);
    /// ```
    fn fail_for_too_small_size_validate() {}

    /// The test ensures that the object must have sufficiently small alignment:
    /// ```compile_fail
    /// use inplace_box::*;
    /// trait Trait {}
    /// #[repr(align(16))]
    /// struct AlignedStruct { b: bool}
    /// impl Trait for AlignedStruct {}
    /// let _b: InplaceBox<dyn Trait, 32> = InplaceBox::new(AlignedStruct { b: false });
    /// ```
    fn fail_for_too_large_alignment() {}

    /// Verification for `fail_for_too_large_alignment()` to check that it
    /// doesn't fail with sufficiently small alignment:
    /// ```
    /// use inplace_box::*;
    /// trait Trait {}
    /// #[repr(align(8))]
    /// struct AlignedStruct { b: bool}
    /// impl Trait for AlignedStruct {}
    /// let _b: InplaceBox<dyn Trait, 32> = InplaceBox::new(AlignedStruct { b: false });
    /// ```
    fn fail_for_too_large_alignment_validate() {}
}
