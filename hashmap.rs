// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[allow(unused_imports)] #[prelude_import] use lrs::prelude::*;
use lrs::{mem};
use lrs::vec::{SVec, Vec};
use lrs::iter::{Iterator, IntoIterator};
use lrs::rc::{Arc};

use ast::{ItemData};

const FACTOR: usize = 2;

pub struct ItemMap {
    buckets: SVec<SVec<(u64, Arc<ItemData>)>>,
    count: usize,
}

impl ItemMap {
    pub fn new() -> ItemMap {
        ItemMap {
            buckets: Vec::new(),
            count: 0,
        }
    }

    fn resize(&mut self) -> Result {
        let new_size = FACTOR * self.count + 1;
        let mut map = ItemMap {
            buckets: try!(Vec::with_capacity(new_size)),
            count: self.count,
        };
        for i in 0..new_size {
            map.buckets.push(Vec::new());
        }
        for element in &*self {
            let hash = (element.0 % new_size as u64) as usize;
            try!(map.buckets[hash].reserve(1));
            map.buckets[hash].push((element.0, element.1.new_ref()));
        }
        mem::replace(self, map);
        Ok(())
    }

    pub fn add(&mut self, id: u64, item: Arc<ItemData>) -> Result {
        if self.count >= FACTOR * self.buckets.len() {
            try!(self.resize());
        }
        let hash = (id % self.buckets.len() as u64) as usize;
        try!(self.buckets[hash].reserve(1));
        self.buckets[hash].push((id, item));
        Ok(())
    }
}

impl<'a> IntoIterator for &'a ItemMap {
    type Item = (u64, &'a Arc<ItemData>);
    type IntoIter = ItemIter<'a>;
    fn into_iter(self) -> ItemIter<'a> {
        ItemIter { items: self, bucket: 0, element: 0, }
    }
}

pub struct ItemIter<'a> {
    items: &'a ItemMap,
    bucket: usize,
    element: usize,
}

impl<'a> Iterator for ItemIter<'a> {
    type Item = (u64, &'a Arc<ItemData>);
    fn next(&mut self) -> Option<(u64, &'a Arc<ItemData>)> {
        if self.bucket >= self.items.buckets.len() {
            return None;
        }
        while self.element >= self.items.buckets[self.bucket].len() {
            self.bucket += 1;
            self.element = 0;
            if self.bucket >= self.items.buckets.len() {
                return None;
            }
        }
        let item = &self.items.buckets[self.bucket][self.element];
        self.element += 1;
        Some((item.0, &item.1))
    }
}
