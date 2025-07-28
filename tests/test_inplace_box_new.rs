use inplace_box::InplaceBox;

trait Foo {}
impl<T> Foo for T {}

trait Bar {}
impl<T> Bar for T {}

/// When creating a nested `InplaceBox`, it should just move the inner
#[test]
fn test_new_with_inplace_box() {
    // Move out
    {
        let box_foo: InplaceBox<dyn Foo, 8> = InplaceBox::new(42u64);
        // Size the same with same trait.
        let _box_foo_2: InplaceBox<dyn Foo, 8> = InplaceBox::new(box_foo);
    }

    // Create new
    {
        let box_foo: InplaceBox<dyn Foo, 8> = InplaceBox::new(42u64);
        // Size different with different trait.
        let _box_bar: InplaceBox<dyn Bar, 16> = InplaceBox::new(box_foo);
    }
}
