// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[allow(unused_imports)] #[prelude_import] use lrs::prelude::*;
use lrs::io::{Write};
use lrs::file::{self, File};
use lrs::{process, env};
use lrs::string::{ByteString, SByteString, ByteStr, AsByteStr};
use lrs::vec::{SVec};

use tree::*;

mod markup;

pub fn create(krate: Crate, base: &str) -> Result {
    let _ = file::create_dir(base, file::Mode::new_directory());
    try!(env::set_cwd(base));

    let mut parts = try!(Vec::with_capacity(1));
    parts.push(&b"lrs"[..]);

    module(&krate.item, &mut parts)
}

fn filename(parts: &[&[u8]]) -> Result<SVec<u8>> {
    let mut res = Vec::new();
    try!(res.push_all(parts[0]));
    for part in &parts[1..] {
        if part.len() > 0 {
            try!(res.push_all(b"$"));
            try!(res.push_all(*part));
        }
    }
    try!(res.push_all(b".html"));

    Ok(res)
}

fn title(parts: &[&[u8]], prefix: &str) -> Result<SByteString> {
    let mut vec = Vec::new();
    try!(vec.push_all(prefix.as_bytes()));
    try!(vec.push_all(parts[0]));
    for part in &parts[1..] {
        if part.len() > 0 {
            try!(vec.push_all(b"::"));
            try!(vec.push_all(*part));
        }
    }
    Ok(ByteString::from_vec(vec))
}

fn html_header<W: Write>(mut w: &mut W, title: &ByteStr) -> Result {
    write!(w, "\
        <html>\
            <head>\
                <title>{}</title>\
                <link rel=\"stylesheet\" type=\"text/css\" href=\"style.css\" />\
            </head>\
            <body>\
            ", title)
}

fn html_footer<W: Write>(mut w: &mut W) -> Result {
    w.write_all(b"\
            </body>\
        </html>\
        ").ignore_ok()
}

fn module<'a>(item: &'a ItemData, parts: &mut SVec<&'a [u8]>) -> Result {
    try!(parts.try_push(item.name.as_ref().unwrap().as_ref()));
    let file = try!(filename(&parts));
    println!("{:?}", file.as_byte_str());
    let mut flags = file::Flags::new();
    flags.set_writable(true);
    flags.set_truncate(true);
    flags.enable_create(file::Mode::new_file());
    let mut file = try!(File::open(&file[..], flags));
    let mut title = try!(title(&parts, "Module "));
    try!(html_header(&mut file, &title));

    try!(write!(file, "<h1>{}</h1>", title));
    try!(markup::short(&mut file, &item.docs.parts));
    try!(markup::description(&mut file, &item.docs.parts));

    let module = match item.inner {
        Item::Module(ref m) => m,
        _ => errexit!("Tried to format a non-module as a module"),
    };

    let mut sub_mods: Vec<_> = Vec::new();

    for item in &module.items {
        match item.inner {
            Item::Module(ref m) => sub_mods.push((item, m)),
            _ => { },
        }
    }

    if sub_mods.len() == 0 {
        return Ok(());
    }

    sub_mods.sort_by(|&(m1,_), &(m2,_)| m1.name.as_ref().unwrap().as_ref()
                                .cmp(m2.name.as_ref().unwrap().as_ref()));

    try!(file.write_all(b"\
        <h2>Modules</h2>\
        <table>\
            <tbody>\
                <tr>\
                    <th>Name</th>\
                    <th>Description</th>\
                </tr>\
                "));

    for &(item, module) in &sub_mods {
        try!(write!(file, "\
            <tr>\
                <td><a href=\"?\">{}</a></td>\
                <td>\
                ", item.name.as_ref().unwrap()));
        try!(markup::short(&mut file, &item.docs.parts));
        try!(file.write_all(b"\
                </td>\
            </tr>\
            "));
    }

    try!(file.write_all(b"\
            </tbody>\
        </table>\
        "));

    try!(html_footer(&mut file));

    Ok(())
}
