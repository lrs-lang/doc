// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[allow(unused_imports)] #[prelude_import] use lrs::prelude::*;
use lrs::io::{Write};
use lrs::string::{AsByteStr, ByteString};
use lrs::util::{memchr};
use lrs::bx::{Box};

use markup::*;

pub fn all<W: Write>(w: &mut W, parts: &[Part]) -> Result {
    for part in parts {
        match *part {
            Part::SectionHeader(n, ref text) => try!(section_header(w, n, text)),
            Part::Block(ref data) => try!(block_data(w, data, false)),
        }
    }
    Ok(())
}

pub fn short<W: Write>(w: &mut W, parts: &[Part]) -> Result {
    for part in parts {
        match *part {
            Part::SectionHeader(1, _) => break,
            Part::SectionHeader(n, ref text) => try!(section_header(w, n, text)),
            Part::Block(ref data) => try!(block_data(w, data, false)),
        }
    }
    Ok(())
}

pub fn field_desc<W: Write>(w: &mut W, parts: &[Part], name: &[u8]) -> Result {
    for part in parts {
        match *part {
            Part::SectionHeader(1, _) => break,
            Part::SectionHeader(_, _) => { },
            Part::Block(ref data) => {
                for attr in &data.attributes {
                    println!("{}", attr.name);
                    if attr.name.trim() == "field" {
                        if let Some(ref a) = attr.args {
                            if a.trim() == name.as_byte_str() {
                                try!(block_data(w, data, true));
                                return Ok(());
                            }
                        }
                    }
                }
            },
        }
    }
    Ok(())
}

fn text_block_is(block: &TextBlock, val: &str) -> bool {
    match block.inner {
        Text::Raw(ref s) if s == val => true,
        _ => false,
    }
}

pub fn description<W: Write>(w: &mut W, parts: &[Part]) -> Result {
    section(w, parts, "Description")
}

pub fn remarks<W: Write>(w: &mut W, parts: &[Part]) -> Result {
    section(w, parts, "Remarks")
}

pub fn see_also<W: Write>(w: &mut W, parts: &[Part]) -> Result {
    section(w, parts, "See also")
}

pub fn section<W: Write>(w: &mut W, parts: &[Part], name: &str) -> Result {
    let pos = parts.find(|p| {
        match *p {
            Part::SectionHeader(1, ref n) if text_block_is(n, name) => true,
            _ => false,
        }
    });
    let pos = match pos {
        Some(p) => p,
        _ => return Ok(()),
    };
    match parts[pos] {
        Part::SectionHeader(1, ref n) => try!(section_header(w, 1, n)),
        _ => { },
    }
    for part in &parts[pos+1..] {
        match *part {
            Part::SectionHeader(1, _) => break,
            Part::SectionHeader(n, ref text) => try!(section_header(w, n, text)),
            Part::Block(ref data) => try!(block_data(w, data, false)),
        }
    }
    Ok(())
}

pub fn section_header<W: Write>(mut w: &mut W, depth: usize,
                                block: &TextBlock) -> Result {

    try!(write!(w, "<h{}>", depth + 1));
    try!(text_block(w, block));
    try!(write!(w, "</h{}>", depth + 1));
    Ok(())
}

pub fn text_block<W: Write>(mut w: &mut W, block: &TextBlock) -> Result {
    let attr = match block.attribute {
        Some(TextAttr::Raw) => "code",
        Some(TextAttr::Bold) => "b",
        _ => "",
    };

    if attr.len() > 0 {
        try!(write!(w, "<{}>", attr));
    }

    try!(text(w, &block.inner));

    if attr.len() > 0 {
        try!(write!(w, "</{}>", attr));
    }

    Ok(())
}

pub fn text<W: Write>(mut w: &mut W, txt: &Text) -> Result {
    match *txt {
        Text::Raw(ref s) => try!(raw(w, s.as_ref())),
        Text::Nested(ref blocks) => {
            for b in blocks {
                try!(text_block(w, b));
            }
        },
        Text::Link(ref l, ref txt) => try!(link(w, l, txt)),
    }
    Ok(())
}

pub fn link<W: Write>(mut w: &mut W, link: &ByteString,
                      txt: &Option<Box<TextBlock>>) -> Result {
    if let Some(ref txt) = *txt {
        try!(write!(w, "<a href=\"{}\">", link));
        try!(text_block(w, txt));
        try!(write!(w, "</a>"));
        return Ok(());
    }
    
    if link.as_ref().starts_with(b"man:") {
        if let Some(p) = memchr(link.as_ref(), b'(') {
            try!(write!(w,
                "<a href=\"http://man7.org/linux/man-pages/man{}/{}.{}.html\">{}</a>",
                link[p+1..link.len()-1], link[4..p], link[p+1..link.len()-1], link[4..]));
            return Ok(());
        }
    }

    if link.as_ref().starts_with(b"lrs") {
        try!(write!(w, "<a href=\"./{}.html\">{}</a>", link, link));
        return Ok(());
    }

    try!(write!(w, "<a href=\"{}\">{}</a>", link, link));
    Ok(())
}

pub fn raw<W: Write>(mut w: &mut W, txt: &[u8]) -> Result {
    for &b in txt {
        match b {
            b'&' => try!(w.write_all(b"&amp;")),
            b'<' => try!(w.write_all(b"&lt;")),
            b'>' => try!(w.write_all(b"&gt;")),
            _ => try!(w.write_all(&[b])),
        };
    }
    Ok(())
}

pub fn block_data<W: Write>(mut w: &mut W, data: &BlockData,
                            show_hidden: bool) -> Result {
    if !show_hidden {
        for attr in &data.attributes {
            let name = attr.name.trim();
            for &aname in &["hidden", "arg", "field"][..] {
                if name == aname {
                    return Ok(());
                }
            }
        }
    }

    match data.inner {
        Block::Grouped(ref blocks) => {
            for data in blocks {
                try!(block_data(w, data, false));
            }
        },
        Block::Code(ref c) => {
            try!(w.write_all(b"<pre>"));
            try!(raw(w, c.as_ref()));
            try!(w.write_all(b"</pre>"));
        },
        Block::List(ref l) => {
            try!(w.write_all(b"<ul>"));
            for el in l {
                try!(w.write_all(b"<li>"));
                match *el {
                    ListEl::Simple(ref b) => {
                        try!(w.write_all(b"<p>"));
                        try!(text_block(w, b));
                        try!(w.write_all(b"</p>"));
                    },
                    ListEl::Complex(ref d) => try!(block_data(w, d, false)),
                }
                try!(w.write_all(b"</li>"));
            }
            try!(w.write_all(b"</ul>"));
        },
        Block::Text(ref t) => {
            try!(w.write_all(b"<p>"));
            try!(text_block(w, t));
            try!(w.write_all(b"</p>"));
        },
        Block::Table(ref t) => {
            try!(w.write_all(b"<table>"));
            for row in t {
                try!(w.write_all(b"<tr>"));
                for col in &row.cols {
                    try!(w.write_all(b"<td>"));
                    match *col {
                        TableCol::Simple(ref t) => try!(text_block(w, t)),
                        TableCol::Complex(ref d) => try!(block_data(w, d, false)),
                    }
                    try!(w.write_all(b"</td>"));
                }
                try!(w.write_all(b"</tr>"));
            }
            try!(w.write_all(b"</table>"));
        },
    };

    Ok(())
}
