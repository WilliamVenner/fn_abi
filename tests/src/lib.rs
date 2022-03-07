#![allow(dead_code)]

#[macro_use] extern crate fn_abi;


#[abi("C")]
extern fn naked_fn_extern() {
	println!("hello world!");
}

#[abi("C")]
fn naked_fn_no_extern() {
	println!("hello world!");
}

#[abi("C")]
extern "cdecl" fn naked_fn_default() {
	println!("hello world!");
}


#[abi("C")]
type TypeAliasNoExtern = fn();
const TYPE_ALIAS_NO_EXTERN: TypeAliasNoExtern = naked_fn_no_extern;

#[abi("C")]
type TypeAliasExtern = extern fn();
const TYPE_ALIAS_EXTERN: TypeAliasExtern = naked_fn_extern;

#[abi("C")]
type TypeAliasDefault = extern "cdecl" fn();
const TYPE_ALIAS_DEFAULT: TypeAliasDefault = naked_fn_default;


#[abi("C")]
const CONST_NO_EXTERN: fn() = naked_fn_no_extern;

#[abi("C")]
const CONST_EXTERN: extern fn() = naked_fn_extern;

#[abi("C")]
const CONST_EXTERN_DEFAULT: extern "cdecl" fn() = naked_fn_default;


#[abi("C")]
static STATIC_NO_EXTERN: fn() = naked_fn_no_extern;

#[abi("C")]
static STATIC_EXTERN: extern fn() = naked_fn_extern;

#[abi("C")]
static STATIC_EXTERN_DEFAULT: extern "cdecl" fn() = naked_fn_default;


#[abi("C")]
extern {
	fn extern_block_fn();
}

#[abi("cdecl")]
extern "C" {
	fn extern_block_fn_with_specified_default();
}

#[test]
fn fake_test() {}