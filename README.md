# Shallow debug

A crate that allows any type to derive a very simple and "shallow" debug impl. The impl will
only print the enum variant, but not the content of the variant. For structs, it will only
print the struct's name, and none of it's field's values.

This is mainly useful for enums when the variant is already useful information. Since none of
the inner values are printed, they don't have to implement `Debug`, so this can also be useful
in highly generic code where you just want a quick and simple way to get debug information.

## Example

```rust
#[derive(ShallowDebug)]
enum MyEnum<A, B, C> {
    A(A),
    B(B),
    C(C),
}

let value: MyEnum<i32, &str, usize> = MyEnum::A(123);
assert_eq!(format!("{value:?}"), "MyEnum::A(..)");
```
