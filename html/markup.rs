// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[allow(unused_imports)] #[prelude_import] use lrs::prelude::*;
use lrs::io::{Write};

use markup::*;

pub fn short<W: Write>(w: &mut W, parts: &[Part]) -> Result {
    for part in parts {
        match *part {
            Part::SectionHeader(1, _) => break,
            Part::SectionHeader(n, ref text) => try!(section_header(w, n, text)),
            Part::Block(ref data) => try!(block_data(w, data)),
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
    let pos = parts.find(|p| {
        match *p {
            Part::SectionHeader(1, ref n) if text_block_is(n, "Description") => true,
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
            Part::Block(ref data) => try!(block_data(w, data)),
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
        Text::Link(ref link, ref txt) => {
            try!(write!(w, "<a href=\"{}\">", link));
            match *txt {
                Some(ref txt) => try!(text_block(w, txt)),
                _ => try!(raw(w, link.as_ref())),
            }
            try!(write!(w, "</a>"));
        },
    }
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

pub fn block_data<W: Write>(mut w: &mut W, block_data: &BlockData) -> Result {
    for attr in &block_data.attributes {
        if &attr.name == "hidden" {
            return Ok(());
        }
    }
    block(w, &block_data.inner)
}

pub fn block<W: Write>(mut w: &mut W, block: &Block) -> Result {
    match *block {
        Block::Grouped(ref blocks) => {
            for data in blocks {
                try!(block_data(w, data));
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
                try!(w.write_all(b"<li><p>"));
                match *el {
                    ListEl::Simple(ref b) => try!(text_block(w, b)),
                    ListEl::Complex(ref d) => try!(block_data(w, d)),
                }
                try!(w.write_all(b"</p></li>"));
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
                        TableCol::Complex(ref d) => try!(block_data(w, d)),
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

//pub fn code<W: Write>(mut w: &mut W, c: &[u8]) -> Result {
//}
