//! # ✨ `fn_abi`
//!
//! A proc attribute macro that sets the ABI/calling convention for the attributed function.
//!
//! ## Example
//!
//! ```rust
//! #[macro_use] extern crate fn_abi;
//!
//! #[abi("fastcall")]
//! extern fn hello_world_fastcall() {
//!     println!("hello world!");
//! }
//!
//! #[cfg_attr(all(target_os = "windows", target_pointer_width = "32"), abi("thiscall"))]
//! #[cfg_attr(all(target_os = "windows", target_pointer_width = "64"), abi("fastcall"))]
//! extern fn hello_world_windows() {
//!     println!("hello world!");
//! }
//! ```

use proc_macro::{TokenStream, TokenTree};
use quote::{quote, ToTokens};
use syn::{Abi, ItemFn, ItemForeignMod, ItemType, ItemStatic, LitStr, TypeBareFn, ItemConst};
use syn_squash::syn_squash;

enum MutableAbi<'a> {
	Option(&'a mut Option<Abi>),
	Required(&'a mut Abi)
}

fn lit_to_abi(lit: LitStr) -> Abi {
	Abi { extern_token: Default::default(), name: Some(lit) }
}

syn_squash! {
	syn_squash_fn => {
		default! => {
			fn abi(&mut self) -> MutableAbi {
				unreachable!();
			}

			fn set_abi(&mut self, lit: LitStr) {
				match self.abi() {
					MutableAbi::Option(opt) => *opt = Some(lit_to_abi(lit)),
					MutableAbi::Required(req) => *req = lit_to_abi(lit)
				}
			}
		};

		ItemFn => {
			fn abi(&mut self) -> MutableAbi {
				MutableAbi::Option(&mut self.sig.abi)
			}
		};

		ItemForeignMod => {
			fn abi(&mut self) -> MutableAbi {
				MutableAbi::Required(&mut self.abi)
			}
		};

		ItemStatic => {
			fn set_abi(&mut self, lit: LitStr) {
				let function = match &mut *self.ty {
					syn::Type::BareFn(function) => function,
					_ => panic!("Only bare function types are supported, please use the macro on a type alias instead"),
				};

				function.abi = Some(lit_to_abi(lit));
			}
		};

		ItemConst => {
			fn set_abi(&mut self, lit: LitStr) {
				let function = match &mut *self.ty {
					syn::Type::BareFn(function) => function,
					_ => panic!("Only bare function types are supported, please use the macro on a type alias instead"),
				};

				function.abi = Some(lit_to_abi(lit));
			}
		};

		ItemType => {
			fn set_abi(&mut self, lit: LitStr) {
				let mut alias = syn::parse::<TypeBareFn>(self.ty.to_token_stream().into()).expect("This type alias does not alias a supported type by fn_abi");
				alias.abi = Some(lit_to_abi(lit));
				self.ty = Box::new(syn::Type::BareFn(alias));
			}
		}
	}
}

#[proc_macro_attribute]
/// # ✨ `fn_abi`
///
/// A proc attribute macro that sets the ABI/calling convention for the attributed function.
///
/// ## Example
///
/// ```rust
/// #[macro_use] extern crate fn_abi;
///
/// #[abi("fastcall")]
/// extern fn hello_world_fastcall() {
///     println!("hello world!");
/// }
///
/// #[cfg_attr(all(target_os = "windows", target_pointer_width = "32"), abi("thiscall"))]
/// #[cfg_attr(all(target_os = "windows", target_pointer_width = "64"), abi("fastcall"))]
/// extern fn hello_world_windows() {
///     println!("hello world!");
/// }
/// ```
pub fn abi(args: TokenStream, input: TokenStream) -> TokenStream {
	macro_rules! must_match {
		($token:expr => $match:path) => {
			if let $match(v) = $token {
				v
			} else {
				panic!("Unexpected {:?}", $token);
			}
		};

		($token:expr => $match:path => $val:literal) => {
			if let $match(v) = $token {
				if v.to_string().as_str() != $val {
					panic!(concat!("Expected a ", $val));
				} else {
					v
				}
			} else {
				panic!("Unexpected {:?}", $token);
			}
		};
	}

	let mut function = syn_squash_fn(input).expect("fn_abi is not supported on this item");

	let mut args = args.into_iter();
	let abi_token = must_match!(args.next().expect("Expected an ABI (invalid argument found)") => TokenTree::Literal);
	let desired_abi = abi_token.to_string();
	let desired_abi = &desired_abi[1..desired_abi.len()-1];
	function.set_abi(LitStr::new(desired_abi, abi_token.span().into()));
	function.into_tokens().into()
}
