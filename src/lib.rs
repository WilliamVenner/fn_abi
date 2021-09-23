#![doc = include_str!("../README.md")]

#[inline]
fn match_target<S: AsRef<str>>(triplet: S) -> bool {
	macro_rules! translation_table {
		($($triplet:literal => $cfg:meta),+) => {{
			#[cfg(not(any($($cfg),+)))] {
				compile_error!("Sorry, but this target triple is not supported by fn_abi yet. See the examples for how to use a custom cfg() directive.");
				return unimplemented!();
			}

			$(
				#[cfg($cfg)] {
					if triplet.as_ref() == $triplet {
						return true;
					}
				}
			)+

			false
		}}
	}

	translation_table!(
		"linux64" => all(target_os = "linux", target_pointer_width = "64"),
		"linux32" => all(target_os = "linux", target_pointer_width = "32"),
		"win32"   => all(target_os = "windows", target_pointer_width = "32"),
		"win64"   => all(target_os = "windows", target_pointer_width = "64"),
		"macos32" => all(target_os = "macos", target_pointer_width = "32"),
		"macos64" => all(target_os = "macos", target_pointer_width = "64"),

		"linux" => target_os = "linux",
		"win"   => target_os = "windows",
		"macos" => target_os = "macos",

		"64" => target_pointer_width = "64",
		"32" => target_pointer_width = "32"
	)
}

use proc_macro::{TokenStream, TokenTree};
use quote::{quote, ToTokens};
use syn::{Abi, ItemFn, ItemForeignMod, ItemType, ItemStatic, LitStr, TypeBareFn};

#[proc_macro_attribute]
#[doc = include_str!("../README.md")]
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

	let mut function = parse_any_mutable_abi(input).expect("fn_abi is not supported on this item");
	let function_abi = function.abi().expect("Missing `extern` keyword in function signature");

	let mut args = args.into_iter().peekable();
	match args.peek() {
		None => panic!("Expected an ABI or target shortcut table (none found)"),
		Some(TokenTree::Literal(_)) => {
			let abi_token = args.next().unwrap();
			let desired_abi = abi_token.to_string();
			let desired_abi = &desired_abi[1..desired_abi.len()-1];
			function.set_abi(Some(LitStr::new(desired_abi, abi_token.span().into())));

			assert!(args.next().is_none(), "Expected an ABI or target shortcut table (invalid argument found)");

			return function.into_tokens();
		}
		_ => while let Some(arg) = args.next() {
			let token = must_match!(arg => TokenTree::Ident).to_string();
			if match_target(token) {
				must_match!(args.next().expect("Expected a =") => TokenTree::Punct => "=");

				let abi_token = must_match!(args.next().expect("Expected a literal") => TokenTree::Literal);
				let desired_abi = abi_token.to_string();
				let desired_abi = &desired_abi[1..desired_abi.len()-1];

				function.set_abi(Some(LitStr::new(desired_abi, abi_token.span().into())));

				return function.into_tokens();
			} else {
				must_match!(args.next().expect("Expected a =") => TokenTree::Punct => "=");
				must_match!(args.next().expect("Expected a literal") => TokenTree::Literal);
				if let Some(arg) = args.next() {
					must_match!(arg => TokenTree::Punct => ",");
					continue;
				} else {
					break;
				}
			}
		}
	};

	if function_abi.name.is_some() {
		function.into_tokens()
	} else {
		panic!("Missing ABI for this target, and no default was specified (e.g. `extern \"Rust\"`)");
	}
}

enum MaybeMutableAbi<'a> {
	Mutable(&'a mut Abi),
	Owned(Abi)
}
impl core::ops::Deref for MaybeMutableAbi<'_> {
	type Target = Abi;

	fn deref(&self) -> &Self::Target {
		match self {
			MaybeMutableAbi::Mutable(borrowed) => *borrowed,
			MaybeMutableAbi::Owned(ref owned) => owned,
		}
	}
}
impl core::ops::DerefMut for MaybeMutableAbi<'_> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		match self {
			MaybeMutableAbi::Mutable(borrowed) => *borrowed,
			MaybeMutableAbi::Owned(ref mut owned) => owned,
		}
	}
}
impl<'a> From<&'a mut Abi> for MaybeMutableAbi<'a> {
	fn from(borrowed: &'a mut Abi) -> Self {
		MaybeMutableAbi::Mutable(borrowed)
	}
}
impl From<Abi> for MaybeMutableAbi<'_> {
	fn from(owned: Abi) -> Self {
		MaybeMutableAbi::Owned(owned)
	}
}
impl Into<Abi> for MaybeMutableAbi<'_> {
	fn into(self) -> Abi {
		if let MaybeMutableAbi::Owned(owned) = self {
			owned
		} else {
			panic!("MaybeMutableAbi::into(Abi) called on non-owned MaybeMutableAbi");
		}
	}
}

trait SynFnIntoTokens {
	fn into_tokens(&self) -> TokenStream;
}
trait SynFn: SynFnIntoTokens {
	fn parse(input: TokenStream) -> Option<Box<dyn SynFn>> where Self: 'static + Sized + syn::parse::Parse {
		syn::parse::<Self>(input).ok().map(|function| Box::new(function) as _)
	}

	fn abi(&mut self) -> Option<MaybeMutableAbi<'_>> {
		unreachable!();
	}

	fn set_abi(&mut self, abi: Option<LitStr>) {
		self.abi().unwrap().name = abi;
	}
}
macro_rules! impl_syn_fn {
	{ $($ty:ty => $code:tt);* } => {
		$(
			impl SynFnIntoTokens for $ty {
				fn into_tokens(&self) -> TokenStream {
					let function: &$ty = self;
					quote!(#function).into()
				}
			}
			impl SynFn for $ty $code
		)*

		fn parse_any_mutable_abi(input: TokenStream) -> Option<Box<dyn SynFn>> {
			$(
				if let Some(function) = <$ty as SynFn>::parse(input.clone()) {
					return Some(function);
				}
			)*
			None
		}
	};
}
impl_syn_fn! {
	ItemFn => {
		fn abi(&mut self) -> Option<MaybeMutableAbi<'_>> {
			self.sig.abi.as_mut().map(Into::into)
		}
	};

	TypeBareFn => {
		fn abi(&mut self) -> Option<MaybeMutableAbi<'_>> {
			self.abi.as_mut().map(Into::into)
		}
	};

	ItemForeignMod => {
		fn abi(&mut self) -> Option<MaybeMutableAbi<'_>> {
			Some((&mut self.abi).into())
		}
	};

	ItemStatic => {
		fn abi(&mut self) -> Option<MaybeMutableAbi<'_>> {
			let function = match &mut *self.ty {
				syn::Type::BareFn(function) => function,
				_ => panic!("Only bare function types are supported, please use the macro on a type alias instead"),
			};
			function.abi.as_mut().map(Into::into)
		}

		fn set_abi(&mut self, lit: Option<LitStr>) {
			let function = match &mut *self.ty {
				syn::Type::BareFn(function) => function,
				_ => unreachable!()
			};
			function.abi.as_mut().unwrap().name = lit;
		}
	};

	ItemType => {
		fn abi(&mut self) -> Option<MaybeMutableAbi<'_>> {
			let alias = syn::parse::<TypeBareFn>(self.ty.to_token_stream().into()).expect("This type alias does not alias a supported type by fn_abi");
			alias.abi.map(Into::into)
		}

		fn set_abi(&mut self, lit: Option<LitStr>) {
			let mut alias = syn::parse::<TypeBareFn>(self.ty.to_token_stream().into()).unwrap();
			alias.abi.as_mut().unwrap().name = lit;
			self.ty = Box::new(syn::Type::BareFn(alias));
		}
	}
}