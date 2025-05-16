// Need to add generic_const_exprs feature in the test too.
// See: https://github.com/rust-lang/rust/issues/133199#issuecomment-2630615573
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]

use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use inplace_box::InplaceBox;

#[test]
fn test_fn_once() {
    trait MyTrait {
        fn method(&self) -> i32;
    }

    struct MyStruct(i32);

    impl MyTrait for MyStruct {
        fn method(&self) -> i32 {
            self.0
        }
    }

    let b = InplaceBox::<dyn MyTrait>::new(MyStruct(42));
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
        let _f = InplaceBox::<dyn MyTrait>::new(v);
    }
    assert_eq!(1, drop_count.load(Ordering::Relaxed), "drop is called");

    let v = MyStruct {
        drop_count: drop_count.clone(),
    };
    {
        let _f = InplaceBox::<dyn MyTrait>::new(v);
    }
    assert_eq!(2, drop_count.load(Ordering::Relaxed), "drop is called");
}
