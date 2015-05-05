// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![crate_type = "bin"]
#![crate_name = "lrs_doc"]
#![feature(plugin, no_std)]
#![plugin(lrs_core_plugin)]
#![no_std]

#[macro_use] extern crate lrs;
mod core { pub use lrs::core::*; }
#[allow(unused_imports)] #[prelude_import] use lrs::prelude::*;

use lrs::{process};
use lrs::file::{File};

#[macro_use] mod macros;
mod json;
mod tree;
mod parse;
mod hashmap;
mod html;
mod markup;
mod passes;

fn main() {
    let mut vec: Vec<_> = Vec::new();
    let file = tryerr!(File::open_read("doc.json"), "Could not open doc.json");
    // tryerr!(vec.read_to_eof(STDIN), "Could not read STDIN");
    tryerr!(vec.read_to_eof(file), "Could not read doc.json");
    let json = tryerr!(json::parse(&vec), "Could not parse JSON");
    let krate = tryerr!(parse::parse(&json), "Could not parse AST");
    passes::run(&krate);

    tryerr!(html::create(krate), "Could not create html");
    // if let Item::Module(ref m) = ast.item.inner {
    //     for item in &m.items {
    //         println!("{:?}", item.name);
    //     }
    // }
}
