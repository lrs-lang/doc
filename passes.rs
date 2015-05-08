// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[allow(unused_imports)] #[prelude_import] use lrs::prelude::*;
use lrs::rc::{Arc};
use lrs::vec::{SVec};
use tree::{self, Walker, ItemData, ResolvedPath, Crate, Impl, Type, Item};
use hashmap::{ItemMap};

pub fn run(krate: &Crate) {
    let mut map = ItemMap::new();

    (CollectItems { map: &mut map }).walk_crate(krate);
    (AddParents { parents: Vec::new() }).walk_crate(krate);
    (LinkTypes { map: &map }).walk_crate(krate);
    (CollectImpls).walk_crate(krate);
}

struct CollectItems<'a> {
    map: &'a mut ItemMap,
}

impl<'a> Walker for CollectItems<'a> {
    fn walk_item_data(&mut self, val: &Arc<ItemData>) {
        self.map.add(val.node, val.new_ref());
        tree::walk_item_data(self, val);
    }
}

struct AddParents {
    parents: SVec<Arc<ItemData>>,
}

impl Walker for AddParents {
    fn walk_item_data(&mut self, val: &Arc<ItemData>) {
        *val.parent.borrow_mut() = self.parents.last().map(|l| l.new_ref());
        self.parents.push(val.new_ref());
        tree::walk_item_data(self, val);
        self.parents.pop();
    }
}

struct LinkTypes<'a> {
    map: &'a ItemMap,
}

impl<'a> Walker for LinkTypes<'a> {
    fn walk_resolved_path(&mut self, val: &ResolvedPath) {
        *val.item.borrow_mut() = self.map.find(val.def_id);
        tree::walk_resolved_path(self, val);
    }
}

struct CollectImpls;

impl Walker for CollectImpls {
    fn walk_item_data(&mut self, val: &Arc<ItemData>) {
        if let Item::Impl(ref i) = val.inner {
            if let Type::ResolvedPath(ref r) = i.for_ {
                if let Some(ref i) = *r.item.borrow() {
                    i.impls.borrow_mut().push(val.new_ref());
                }
            }
        }
        tree::walk_item_data(self, val);
    }
}
