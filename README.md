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

## Shortcuts

`fn_abi` supports a number of target shortcuts that you can use.

```rust
#[abi(
	linux32 = "C",
	linux64 = "C",
	win32 = "thiscall",
	win64 = "fastcall"
)]
extern fn hello_world() {
	println!("hello world!");
}
```

### Supported Targets

Currently, the macro will translate the following targets to their respective `cfg` attributes:

| Shorthand | Expansion |
|:---:|:---:|
| `linux`   | `cfg(target_os = "linux")` |
| `win`     | `cfg(target_os = "linux")` |
| `macos`   | `cfg(target_os = "windows")` |
| `linux32` | `cfg(all(target_os = "linux", target_pointer_width = "32"))` |
| `linux64` | `cfg(all(target_os = "linux", target_pointer_width = "32"))` |
| `win32`   | `cfg(all(target_os = "windows", target_pointer_width = "32"))` |
| `win64`   | `cfg(all(target_os = "windows", target_pointer_width = "32"))` |
| `macos32` | `cfg(all(target_os = "macos", target_pointer_width = "32"))` |
| `macos64` | `cfg(all(target_os = "macos", target_pointer_width = "32"))` |

Please feel free to contribute if you find a target that is not supported.
