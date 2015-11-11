// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::io::{Write};

use html::{Formatter};
use tree::*;

impl Formatter {
    pub fn typedef(&mut self, _item: &ItemData, _typedef: &Typedef) -> Result {
        let mut file: Vec<_> = Vec::new();

        try!(self.head(&mut file, "Typedef "));
        try!(self.h1(&mut file, "Typedef "));

        try!(self.foot(&mut file));

        try!(try!(self.file()).write_all(&file));
        Ok(())
    }
}
