#![crate_type = "lib"]
#![feature(no_std, lang_items)]
#![no_std]

#[lang = "sized"]
trait Sized { }

#[lang = "copy"]
trait Copy { }

pub struct X;

pub const Y: usize = 22;

pub fn f(x: &X) { }

