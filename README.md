# InplaceBox

`InplaceBox` is a Rust library that provides a stack-allocated container similar to `Box`, but without heap allocation. It stores data inline within a fixed-size buffer.

## Features

- Store values on the stack without heap allocation
- Support for dynamically sized types (like trait objects)
- Similar interface to `Box` with `Deref` and `DerefMut` implementations

## Usage

```rust
use inplace_box::InplaceBox;

trait MyTrait {
    fn method(&self) -> i32;
}

struct MyStruct(i32);

impl MyTrait for MyStruct {
    fn method(&self) -> i32 {
        self.0
    }
}

fn main() {
    let inplace_box = InplaceBox::<dyn MyTrait>::new(MyStruct(42));
    assert_eq!(inplace_box.method(), 42);
}
```

## Requirements

This crate requires the nightly Rust compiler as it uses the following unstable features:
- `generic_const_exprs`
- `ptr_metadata`
- `unsize`
- `coerce_unsized`

## How It Works

`InplaceBox` uses Rust's advanced type system features to:
1. Store the data in a fixed-size inline buffer
2. Store vtable metadata for trait objects
3. Implement smart pointer traits for seamless usage

It's ideal for embedded systems, performance-critical code, or situations where heap allocations should be avoided.

## License

MIT

## Contributions

Contributions are welcome! Please feel free to submit a Pull Request.


