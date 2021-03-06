// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io::{Write};
use std::string::{ByteStr};
use std::util::{memchr};
use std::bx::{Box};

use markup::*;

pub fn all<W: Write>(w: &mut W, parts: &[Part]) -> Result {
    for part in parts {
        match *part {
            Part::SectionHeader(n, ref text) => try!(section_header(w, n, text, false)),
            Part::Block(ref data) => try!(block_data(w, data, false)),
        }
    }
    Ok(())
}

pub fn short<W: Write>(w: &mut W, parts: &[Part]) -> Result {
    for part in parts {
        match *part {
            Part::SectionHeader(1, _) => break,
            Part::SectionHeader(n, ref text) => try!(section_header(w, n, text, false)),
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
                    if attr.name.as_str().trim() == "field" {
                        if let Some(ref a) = attr.args {
                            if a.as_str().trim() == name {
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

pub fn return_value<W: Write>(w: &mut W, parts: &[Part]) -> Result {
    for part in parts {
        match *part {
            Part::SectionHeader(1, _) => break,
            Part::SectionHeader(_, _) => { },
            Part::Block(ref data) => {
                for attr in &data.attributes {
                    if attr.name.as_str().trim() == "return_value" {
                        try!(block_data(w, data, true));
                        return Ok(());
                    }
                }
            },
        }
    }
    Ok(())
}

pub fn has_return_value(parts: &[Part]) -> bool {
    for part in parts {
        match *part {
            Part::SectionHeader(1, _) => break,
            Part::SectionHeader(_, _) => { },
            Part::Block(ref data) => {
                for attr in &data.attributes {
                    if attr.name.as_str().trim() == "return_value" {
                        return true;
                    }
                }
            },
        }
    }
    false
}

pub fn arg_desc<W: Write>(w: &mut W, parts: &[Part], name: &ByteStr) -> Result {
    for part in parts {
        match *part {
            Part::SectionHeader(1, _) => break,
            Part::SectionHeader(_, _) => { },
            Part::Block(ref data) => {
                for attr in &data.attributes {
                    if attr.name.as_str().trim() == "argument" {
                        if let Some(ref a) = attr.args {
                            if a.as_str().trim() == name {
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
        Text::Raw(ref s) if s.as_str() == val => true,
        _ => false,
    }
}

pub fn description<W: Write>(w: &mut W, parts: &[Part]) -> Result {
    section(w, parts, "Description", false)
}

pub fn remarks<W: Write>(w: &mut W, parts: &[Part]) -> Result {
    section(w, parts, "Remarks", true)
}

pub fn examples<W: Write>(w: &mut W, parts: &[Part]) -> Result {
    section(w, parts, "Examples", true)
}

pub fn see_also<W: Write>(w: &mut W, parts: &[Part]) -> Result {
    section(w, parts, "See also", false)
}

pub fn section<W: Write>(w: &mut W, parts: &[Part], name: &str, info: bool) -> Result {
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
        Part::SectionHeader(1, ref n) => try!(section_header(w, 1, n, info)),
        _ => { },
    }
    for part in &parts[pos+1..] {
        match *part {
            Part::SectionHeader(1, _) => break,
            Part::SectionHeader(n, ref text) => try!(section_header(w, n, text, false)),
            Part::Block(ref data) => try!(block_data(w, data, false)),
        }
    }
    Ok(())
}

pub fn section_header<W: Write>(mut w: &mut W, depth: usize, block: &TextBlock,
                                info: bool) -> Result {

    try!(write!(w, "<h{}>", depth + 1));
    try!(text_block(w, block));
    try!(write!(w, "</h{}>", depth + 1));
    if info {
        try!(w.write_all(br#"<p class="info_head">This section is informative.</p>"#));
    }
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

pub fn link<W: Write>(mut w: &mut W, link: &Vec<u8>,
                      txt: &Option<Box<TextBlock>>) -> Result {
    try!(w.write_all(b"<a href=\""));

    if link.starts_with(b"man:") {
        if let Some(p) = memchr(link.as_ref(), b'(') {
            try!(write!(w,
                "http://man7.org/linux/man-pages/man{}/{}.{}.html\">",
                link.as_str()[p+1..link.len()-1], link.as_str()[4..p],
                link.as_str()[p+1..link.len()-1]));
            match *txt {
                Some(ref txt) => { try!(text_block(w, txt)); }
                _ => { try!(w.write_all(link[4..].as_ref())); }
            }
            try!(w.write_all(b"</a>"));
            return Ok(());
        }
    }

    if link.starts_with(b"lrs") {
        try!(write!(w, "./{}.html\">", link.as_str()));
        match *txt {
            Some(ref txt) => { try!(text_block(w, txt)); }
            _ => { try!(w.write_all(link.as_ref())); }
        }
        try!(w.write_all(b"</a>"));
        return Ok(());
    }

    try!(write!(w, "{}\">", link.as_str()));
    match *txt {
        Some(ref txt) => { try!(text_block(w, txt)); }
        _ => { try!(w.write_all(link.as_ref())); }
    }
    try!(w.write_all(b"</a>"));
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
            let name = attr.name.as_str().trim();
            for &aname in &["hidden", "argument", "return_value", "field"][..] {
                if name == aname {
                    return Ok(());
                }
            }
        }
    }

    let is_info = data.attributes.find(|a| a.name.as_str().trim() == "info").is_some();
    if is_info {
        try!(w.write_all(br#"<div class="informative">"#));
        try!(w.write_all(br#"<p class="info_head">This block is informative.</p>"#));
    }

    let is_quote = data.attributes.find(|a| a.name.as_str().trim() == "quote").is_some();
    if is_quote {
        try!(w.write_all(b"<blockquote>"));
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

    if is_quote {
        try!(w.write_all(b"</blockquote>"));
    }

    if is_info {
        try!(w.write_all(b"</div>"));
    }

    Ok(())
}
