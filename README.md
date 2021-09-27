[![crates.io](https://img.shields.io/crates/v/fn_abi.svg)](https://crates.io/crates/fn_abi)

# ✨ `fn_abi`

A proc attribute macro that sets the ABI/calling convention for the attributed function.

## Example

```rust
#[macro_use]
extern crate fn_abi;

#[abi("fastcall")]
extern fn hello_world() {
    println!("hello world!");
}

#[cfg_attr(all(target_os = "windows", target_pointer_width = "32"), abi("thiscall"))]
#[cfg_attr(all(target_os = "windows", target_pointer_width = "64"), abi("fastcall"))]
extern fn hello_world() {
    println!("hello world!");
}
```