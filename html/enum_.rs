// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[allow(unused_imports)] #[prelude_import] use lrs::prelude::*;

use html::{Formatter};
use markup::{Document};
use tree::*;

impl Formatter {
    pub fn enum_(&mut self, _: &Enum, _: &Document) -> Result {
        let mut file = try!(self.file());

        try!(self.head(&mut file, "Enum "));
        try!(self.h1(&mut file, "Enum "));

        try!(self.foot(&mut file));
        Ok(())
    }

}
