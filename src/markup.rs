// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! lrs_doc markup language for doc comments
//!
//! . stands for any character except '\n'
//!
//! =====================================================================================
//!
//! Document structure
//! ------------------
//!
//! $             <- '\n'
//!
//! SimpleListEl  <- '* ' .* $ ('  ' .* $)*
//! BlockListEl   <- '**' $ Block
//! ListEl        <- SimpleListEl / BlockListEl
//! ListBlock     <- ListEl+
//!
//! TableDelim    <- '|===' $
//! ColumnText    <- (!'|' ('\\\\' / '\\|' / .))*
//! SimpleRow     <- ('|' ColumnText)+ $
//! Row           <- (SimpleRow / (!$ Block))* $
//! TableBlock    <- TableDelim ($* !TableDelim Row)* $* TableDelim?
//!
//! TextBlock     <- (.+ $)* $?
//!
//! CodeDelim     <- '----' $
//! CodeBlock     <- CodeDelim (!CodeDelim .* $)* CodeDelim?
//!
//! GroupStart    <- '{' $ 
//! GroupEnd      <- '}' $
//! GroupedBlock  <- GroupStart ($* !GroupEnd Block)* $* GroupEnd?
//!
//! Attribute     <- '[' .* ']' $
//! Block         <- Attribute* (GroupedBlock / CodeBlock / TableBlock / ListBlock /
//!                              TextBlock)
//!
//! SectionHeader <- '='+ ' ' .* $
//!
//! VarName       <- [a-zA-Z_]+
//! VarDef        <- ':' VarName ': ' .* $
//!
//! Document      <- ($* (SectionHeader / VarDef / Block))*
//!
//! =====================================================================================
//!
//! Text structure
//! --------------
//!
//! RawDelim       <- '`'
//! BoldDelim      <- '*'
//! SubstStart     <- '{'
//! SubstEnd       <- '}'
//! LinkEnd        <- ']'
//! LinkStart      <- 'link:'
//!
//! EscapeSequence <- '\\' ('\\' / RawDelim / BoldDelim / SubstStart / LinkEnd /
//!                         LinkStart)
//!
//! UnnamedLink    <- 'link:' (!' ' !'[' .)+
//! LinkText       <- (!']' ('\\]' / .))*
//! NamedLink      <- UnnamedLink '[' LinkText ']'
//! Link           <- NamedLink / UnnamedLink
//!
//! Raw            <- RawDelim (!'`' ('\\`' / '\\\\' / .))* RawDelim?
//! Bold           <- BoldDelim (!'*' ('\\*' / '\\\\' / .))* BoldDelim?
//!
//! Text           <- (EscapeSequenc / Bold / Raw / Link / .)*
//!
//! 
//! This is recursively applied to link/ref text and the content of bold but not raw.

use std::{mem};
use std::bx::{Box};
use std::util::{memchr};
use std::string::{ByteString};
use std::vec::{Vec};
use std::io::{BufRead};

pub struct Document {
    pub parts: Vec<Part>,
}

pub enum Part {
    SectionHeader(usize, TextBlock),
    Block(BlockData),
}

pub struct BlockData {
    pub attributes: Vec<Attribute>,
    pub inner: Block,
}

pub struct Attribute {
    pub name: ByteString,
    pub args: Option<ByteString>,
}

pub enum Block {
    Grouped(Vec<BlockData>),
    Code(ByteString),
    List(Vec<ListEl>),
    Table(Vec<TableRow>),
    Text(TextBlock),
}

pub enum ListEl {
    Simple(TextBlock),
    Complex(BlockData),
}

pub struct TableRow {
    pub cols: Vec<TableCol>,
}

pub enum TableCol {
    Simple(TextBlock),
    Complex(BlockData),
}

pub struct TextBlock {
    pub attribute: Option<TextAttr>,
    pub inner: Text,
}

pub enum Text {
    Raw(ByteString),
    Nested(Vec<TextBlock>),
    Link(ByteString, Option<Box<TextBlock>>),
}

pub enum TextAttr {
    Raw,
    Bold,
}

pub fn parse(input: &[u8]) -> Result<Document> {
    let mut parser = DocParser {
        r: input,
        eof: false,
        next: None,

        parts: Vec::new(),
        vars: Vec::new(),
    };
    try!(parser.document());
    Ok(Document { parts: parser.parts })
}






struct DocParser<R: BufRead> {
    r: R,
    eof: bool,
    next: Option<Vec<u8>>,

    parts: Vec<Part>,
    vars: Vec<(Vec<u8>, Vec<u8>)>,
}

impl<R: BufRead> DocParser<R> {
    fn peek_line(&mut self) -> Result<&[u8]> {
        if self.next.is_none() {
            self.next = Some(try!(self.next_line()));
        }
        Ok(self.next.as_ref().unwrap())
    }

    fn next_line(&mut self) -> Result<Vec<u8>> {
        if self.next.is_some() {
            return Ok(self.next.take().unwrap());
        }

        let mut buf = Vec::new();

        loop {
            let n = try!(self.r.copy_until(&mut buf, b'\n'));
            if n > 0 {
                if buf[buf.len() - 1] == b'\n' {
                    buf.pop();
                }
                if n > 1 && buf[buf.len() - 1] == b'\\' {
                    buf.pop();
                    continue;
                }
            } else {
                self.eof = true;
            }
            break;
        }

        Ok(buf)
    }

    /// Document <- $* ((SectionHeader / Block) $*)*
    fn document(&mut self) -> Result {
        while !self.eof {
            try!(self.blank_lines());

            if self.eof {
                break;
            }

               try!(self.section_header())
            || try!(self.var_def())
            || try!(self.block());
        }

        Ok(())
    }

    fn blank_lines(&mut self) -> Result {
        while try!(self.peek_line()).len() == 0 && !self.eof {
            try!(self.next_line());
        }
        Ok(())
    }

    /// SectionHeader <- '='+ ' ' .* $
    ///
    /// = Section header
    /// == Level 2 section header
    fn section_header(&mut self) -> Result<bool> {
        let len = {
            let line = try!(self.peek_line());
            if line.len() == 0 || line[0] != b'=' { return Ok(false); }
            let mut len = 0;
            for i in 1..line.len() {
                if line[i] == b'=' { continue; }
                if line[i] == b' ' { len = i; break; }
                return Ok(false);
            }
            if len == 0 { return Ok(false); }
            len
        };
        let line = try!(self.next_line());
        let text = try!(TextParser::all_in_one(&line[len+1..], &self.vars));
        try!(self.parts.reserve(1));
        self.parts.push(Part::SectionHeader(len, text));
        Ok(true)
    }

    /// VarName <- [a-zA-Z_]+
    /// VarDef  <- ':' VarName ': ' .* $
    fn var_def(&mut self) -> Result<bool> {
        let end = {
            let line = try!(self.peek_line());
            if line.len() < 3 || line[0] != b':' { return Ok(false); }
            let mut i = 1;
            while i < line.len() {
                match line[i] {
                    b'a'...b'z' | b'A'...b'Z' | b'_' => { },
                    b':' => break,
                    _ => return Ok(false),
                }
                i += 1;
            }
            if i + 1 >= line.len() || line[i+1] != b' ' { return Ok(false); }
            i
        };

        let line = try!(self.next_line());
        let name = try!(line[1..end].to_owned());
        let val = try!(line[end+2..].to_owned());
        try!(self.vars.reserve(1));
        self.vars.push((name, val));
        Ok(true)
    }

    /// Block <- Attribute* (GroupedBlock / CodeBlock / ListBlock / TextBlock)
    fn block(&mut self) -> Result<bool> {
        let attributes = try!(self.attributes());

           try!(self.grouped_block())
        || try!(self.code_block())
        || try!(self.table_block())
        || try!(self.list_block())
        || try!(self.text_block())
        ;

        let mut block = self.pop_block();
        block.attributes = attributes;
        self.parts.push(Part::Block(block));

        Ok(true)
    }

    fn pop_block(&mut self) -> BlockData {
        match self.parts.pop().unwrap() {
            Part::Block(b) => b,
            _ => abort!(),
        }
    }

    fn attributes(&mut self) -> Result<Vec<Attribute>> {
        let mut vec = Vec::new();
        while !self.eof {
            if let Some(a) = try!(self.attribute()) {
                try!(self.next_line());
                try!(vec.reserve(1));
                vec.push(a);
            } else {
                break;
            }
        }
        Ok(vec)
    }

    /// Attribute <- '[' .* ']' $
    ///
    /// [hidden]
    /// [arg, hurr_durr_im_an_arg]
    /// Description of the hurr durr arg that is not shown in the output.
    fn attribute(&mut self) -> Result<Option<Attribute>> {
        let mut line = try!(self.peek_line());

        if line.len() == 0 || line[0] != b'[' || line[line.len()-1] != b']' {
            return Ok(None);
        }
        line = &line[1..line.len()-1];

        if let Some(pos) = memchr(line, b',') {
            let name = ByteString::from_vec(try!(line[..pos].to_owned()));
            let args = ByteString::from_vec(try!(line[pos+1..].to_owned()));
            Ok(Some(Attribute { name: name, args: Some(args) }))
        } else {
            let name = ByteString::from_vec(try!(line.to_owned()));
            Ok(Some(Attribute { name: name, args: None }))
        }
    }

    /// GroupStart   <- '{' $ 
    /// GroupEnd     <- '}' $
    /// GroupedBlock <- GroupStart ($* !GroupEnd Block)* $* GroupEnd?
    ///
    /// {
    /// this is
    ///
    /// a block made out of
    ///
    /// multiple text blocks
    ///
    /// }
    fn grouped_block(&mut self) -> Result<bool> {
        if try!(self.peek_line()) != &b"{"[..] {
            return Ok(false);
        }

        // Discard GroupStart
        self.next_line();

        let mut blocks = Vec::new();
        loop {
            try!(self.blank_lines());

            if self.eof || try!(self.peek_line()) == &b"}"[..] {
                break;
            }

            // Block always parses
            try!(self.block());

            let block = self.pop_block();
            try!(blocks.reserve(1));
            blocks.push(block);
        }

        // Discard GroupEnd if it exists.
        try!(self.next_line());

        let block = BlockData { attributes: Vec::new(), inner: Block::Grouped(blocks) };
        self.parts.push(Part::Block(block));
        Ok(true)
    }

    /// CodeDelim <- '----' $
    /// CodeBlock <- CodeDelim (!CodeDelim .* $)* CodeDelim?
    fn code_block(&mut self) -> Result<bool> {
        if try!(self.peek_line()) != &b"----"[..] {
            return Ok(false);
        }
        
        // Discard CodeDelim
        try!(self.next_line());

        let mut code = Vec::new();

        while !self.eof {
            let line = try!(self.next_line());
            if &line == &b"----"[..] {
                break;
            }
            try!(code.push_all(&line));
            try!(code.push_all(b"\n"));
        }

        // Pop the last \n
        code.pop();

        let block = BlockData {
            attributes: Vec::new(),
            inner: Block::Code(ByteString::from_vec(code)),
        };

        try!(self.parts.reserve(1));
        self.parts.push(Part::Block(block));
        Ok(true)
    }

    /// TableDelim <- '|===' $
    /// ColumnText <- (!'|' ('\\\\' / '\\|' / .))*
    /// SimpleRow  <- ('|' ColumnText)+ $
    /// Row        <- (SimpleRow / (!$ Block))* $
    /// TableBlock <- TableDelim ($* !TableDelim Row)* $* TableDelim?
    fn table_block(&mut self) -> Result<bool> {
        if try!(self.peek_line()) != "|===" {
            return Ok(false);
        }

        // Discard TableDelim
        try!(self.next_line());

        let mut rows = Vec::new();

        'table: loop {
            try!(self.blank_lines());

            if self.eof || try!(self.peek_line()) == "|===" {
                break 'table;
            }

            let mut cols = Vec::new();

            'row: loop {
                if try!(self.peek_line()) == "" {
                    break 'row;
                }

                if try!(self.peek_line())[0] == b'|' {
                    let line = try!(self.next_line());

                    // simple row
                    let mut col_start = 1;
                    let mut i = 1;
                    while i < line.len() {
                        // pass escaped \\ and | to the next level
                        if i + 1 < line.len() && line[i] == b'\\' {
                            match line[i+1] {
                                b'\\' | b'|' => { i += 2; continue; }
                                _ => { },
                            }
                        }
                        if line[i] == b'|' {
                            let text = try!(TextParser::all_in_one(&line[col_start..i],
                                                                   &self.vars));
                            col_start = i + 1;
                            try!(cols.reserve(1));
                            cols.push(TableCol::Simple(text));
                        }
                        i += 1;
                    }
                    // The last column
                    let text = try!(TextParser::all_in_one(&line[col_start..i],
                                                           &self.vars));
                    try!(cols.reserve(1));
                    cols.push(TableCol::Simple(text));
                } else {
                    // block row
                    try!(self.block());
                    let block = self.pop_block();
                    try!(cols.reserve(1));
                    cols.push(TableCol::Complex(block));
                }
            }

            try!(rows.reserve(1));
            rows.push(TableRow { cols: cols });
        }

        // Discard trailing TableDelim if any
        try!(self.next_line());

        let block = BlockData {
            attributes: Vec::new(),
            inner: Block::Table(rows),
        };

        try!(self.parts.reserve(1));
        self.parts.push(Part::Block(block));
        Ok(true)
    }

    /// SimpleListEl <- '* ' .* $ ('  ' .* $)*
    /// BlockListEl  <- '**' $ Block
    /// ListEl       <- SimpleListEl / BlockListEl
    /// ListBlock    <- ListEl+
    fn list_block(&mut self) -> Result<bool> {
        let mut list = Vec::new();

        loop {
            let simple = {
                let line = try!(self.peek_line());
                if line.len() < 2 { break; }
                if line == &b"**"[..] {
                    false
                } else if &line[..2] == &b"* "[..] {
                    true
                } else {
                    break;
                }
            };
            let line = try!(self.next_line());
            let el = if simple {
                let mut buf: Vec<_> = try!(line[2..].to_owned());
                while try!(self.peek_line()).starts_with(b"  ") {
                    let line = try!(self.next_line());
                    try!(buf.push_all(b" "));
                    try!(buf.push_all(&line[2..]));
                }
                let text = try!(TextParser::all_in_one(&buf, &self.vars));
                ListEl::Simple(text)
            } else {
                try!(self.block());
                ListEl::Complex(self.pop_block())
            };
            try!(list.reserve(1));
            list.push(el);
        }

        if list.len() == 0 {
            return Ok(false);
        }

        let block = BlockData { attributes: Vec::new(), inner: Block::List(list) };
        try!(self.parts.reserve(1));
        self.parts.push(Part::Block(block));
        Ok(true)
    }

    /// TextBlock <- (.+ $)* $?
    fn text_block(&mut self) -> Result<bool> {
        let mut text: Vec<_> = Vec::new();

        loop {
            let line = try!(self.next_line());
            if line.len() == 0 {
                break;
            }
            try!(text.push_all(&line));
            try!(text.push_all(b" "));
        }

        // Pop the last space
        text.pop();

        let text = try!(TextParser::all_in_one(&text, &self.vars));

        let block = BlockData {
            attributes: Vec::new(),
            inner: Block::Text(text),
        };

        try!(self.parts.reserve(1));
        self.parts.push(Part::Block(block));

        Ok(true)
    }
}

struct TextParser<'a> {
    text: &'a [u8],
    current: Option<Vec<u8>>,
    past: Vec<TextBlock>,
}

impl<'a> TextParser<'a> {
    fn all_in_one(text: &[u8], vars: &[(Vec<u8>, Vec<u8>)]) -> Result<TextBlock> {
        let vec = try!(TextParser::subst(text, vars));
        TextParser::parse(&vec)
    }

    fn subst(text: &[u8], vars: &[(Vec<u8>, Vec<u8>)]) -> Result<Vec<u8>> {
        let mut text: Vec<_> = try!(text.to_owned());
        let mut next = try!(Vec::with_capacity(text.len()));
        loop {
            let mut did_substitute = false;

            let mut i = 0;
            'outer: while i < text.len() {
                // Pass \\\\ and \\{ without changes.
                if i+1 < text.len() && text[i] == b'\\' {
                    match text[i+1] {
                        b'\\' | b'{' => {
                            try!(next.push_all(&text[i..i+2]));
                            i += 2;
                            continue;
                        }
                        _ => { },
                    }
                }

                // Check if we've found a variable
                if text[i] == b'{' {
                    let mut j = i + 1;
                    let mut looks_like_var = false;
                    while j < text.len() {
                        match text[j] {
                            b'a'...b'z' | b'A'...b'Z' | b'_' => { },
                            b'}' => { looks_like_var = j > i + 1; break; }
                            _ => break,
                        }
                        j += 1;
                    }
                    if looks_like_var {
                        for &(ref var, ref sub) in vars {
                            if var == &text[i+1..j] {
                                did_substitute = true;
                                try!(next.push_all(sub));
                                i = j + 1;
                                continue 'outer;
                            }
                        }
                    }
                }

                // Nope
                try!(next.reserve(1));
                next.push(text[i]);
                i += 1;
            }

            mem::swap(&mut text, &mut next);
            next.truncate(0);
 
            if !did_substitute {
                break;
            }
        }

        Ok(text)
    }

    fn parse(text: &[u8]) -> Result<TextBlock> {
        let mut text = TextParser {
            text: &text,
            current: None,
            past: Vec::new(),
        };

        while text.text.len() > 0 {
            let res =    try!(text.escape_sequence())
                      || try!(text.bold())
                      || try!(text.raw())
                      || try!(text.link())
                      ;
            if !res {
                try!(text.append_raw(&text.text[0..1]));
                text.text.consume(1);
            }
        }

        let (attribute, block) = match (text.past.len(), text.current) {
            (0, Some(c)) => (None, Text::Raw(ByteString::from_vec(c))),
            (0, None   ) => (None, Text::Raw(ByteString::new())),
            (1, None   ) => {
                let TextBlock { attribute, inner } = text.past.pop().unwrap();
                (attribute, inner)
            },
            (_, Some(c)) => {
                let raw = ByteString::from_vec(c);
                try!(text.past.reserve(1));
                text.past.push(TextBlock { attribute: None, inner: Text::Raw(raw) });
                (None, Text::Nested(text.past))
            },
            (_, None   ) => (None, Text::Nested(text.past)),
        };

        Ok(TextBlock { attribute: attribute, inner: block })
    }

    fn escape_sequence(&mut self) -> Result<bool> {
        if self.text.len() < 2 { return Ok(false); }
        if self.text[0] != b'\\' { return Ok(false); }
        let len = if self.text[1] == b'\\' {
            1
        } else if self.text[1] == b'`' {
            1
        } else if self.text[1] == b'*' {
            1
        } else if self.text[1] == b'{' {
            1
        } else if self.text[1] == b']' {
            1
        } else if self.text[1] == b'|' {
            1
        } else if self.text[1..].starts_with(b"link:") {
            b"link:".len()
        } else {
            return Ok(false)
        };
        try!(self.append_raw(&self.text[1..1+len]));
        self.text.consume(len + 1);
        Ok(true)
    }

    fn append_raw(&mut self, r: &[u8]) -> Result {
        if self.current.is_none() { self.current = Some(Vec::new()); }
        try!(self.current.as_mut().unwrap().push_all(r));
        Ok(())
    }

    fn finish_raw(&mut self) -> Result {
        if let Some(c) = self.current.take() {
            let raw = ByteString::from_vec(c);
            try!(self.past.reserve(1));
            self.past.push(TextBlock { attribute: None, inner: Text::Raw(raw) });
        }
        Ok(())
    }

    fn bold(&mut self) -> Result<bool> {
        self.special(b'*', TextAttr::Bold, true)
    }

    fn raw(&mut self) -> Result<bool> {
        self.special(b'`', TextAttr::Raw, false)
    }

    fn special(&mut self, c: u8, attr: TextAttr, cont: bool) -> Result<bool> {
        if self.text[0] != c { return Ok(false); }
        let mut text: Vec<_> = Vec::new();
        let mut i = 1;
        while i < self.text.len() {
            if self.text[i] == c { break; }
            if self.text[i] == b'\\' && i + 1 < self.text.len() { 
                if self.text[i+1] == b'\\' || self.text[i+1] == c {
                    i += 1;
                }
            }
            try!(text.reserve(1));
            text.push(self.text[i]);
            i += 1;
        }
        self.text.consume(i + 1);
        self.finish_raw();
        let block = if cont {
            try!(TextParser::parse(&text))
        } else {
            TextBlock { attribute: None, inner: Text::Raw(ByteString::from_vec(text)) }
        };
        let mut block = if block.attribute.is_none() {
            block
        } else {
            let mut vec = try!(Vec::with_capacity(1));
            vec.push(block);
            TextBlock { attribute: None, inner: Text::Nested(vec) }
        };
        block.attribute = Some(attr);
        try!(self.past.reserve(1));
        self.past.push(block);
        Ok(true)
    }

    fn link(&mut self) -> Result<bool> {
        if !self.text.starts_with(b"link:") { return Ok(false); }
        let mut i = b"link:".len();
        while i < self.text.len() && self.text[i] != b' ' && self.text[i] != b'[' {
            i += 1;
        }

        let mut j = i + 1;
        let mut link_text = None;
        if i < self.text.len() && self.text[i] == b'[' {
            let mut link_text_: Vec<_> = Vec::new();
            while j < self.text.len() {
                if j+1 < self.text.len() && self.text[j] == b'\\' {
                    match self.text[j+1] {
                        b'\\' | b']' => {
                            try!(link_text_.push_all(&self.text[j+1..j+2]));
                            j += 2;
                            continue;
                        },
                        _ => { },
                    }
                }
                if self.text[j] == b']' { break; }
                try!(link_text_.reserve(1));
                link_text_.push(self.text[j]);
                j += 1;
            }
            if j < self.text.len() {
                link_text = Some(link_text_);
            }
        }

        self.finish_raw();

        let link = ByteString::from_vec(try!(self.text[b"link:".len()..i].to_owned()));
        let link_text = match link_text {
            Some(t) => Some(try!(Box::new()).set(try!(TextParser::parse(&t)))),
            _ => None,
        };

        if link_text.is_some() {
            self.text.consume(j + 1);
        } else {
            self.text.consume(i);
        }

        try!(self.past.reserve(1));
        self.past.push(TextBlock { attribute: None, inner: Text::Link(link, link_text) });
        Ok(true)
    }
}
