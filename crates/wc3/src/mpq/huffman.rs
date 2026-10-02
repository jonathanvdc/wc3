//! Blizzard adaptive Huffman decoder, adapted from stormlib-rs 0.0.3.
//! Copyright (c) 2026 notOrrytrout. See licenses/stormlib-rs-MIT.txt.
//! Uses format-specific initial weights and LSB-first bit packing.

use super::{huffman_tables::WEIGHT_TABLES, Error};

const HUFF_ITEM_COUNT: usize = 0x203;
const HUFF_DECOMPRESS_ERROR: u32 = 0x1FF;

#[derive(Clone, Copy, Debug, Default)]
struct InputStream<'a> {
    data: &'a [u8],
    pos: usize,
    bit_buffer: u32,
    bit_count: u32,
}

impl<'a> InputStream<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            bit_buffer: 0,
            bit_count: 0,
        }
    }

    fn get1bit(&mut self) -> Option<u32> {
        if self.bit_count == 0 {
            if self.pos >= self.data.len() {
                return None;
            }
            self.bit_buffer = self.data[self.pos] as u32;
            self.pos += 1;
            self.bit_count = 8;
        }
        let bit = self.bit_buffer & 0x01;
        self.bit_buffer >>= 1;
        self.bit_count -= 1;
        Some(bit)
    }

    fn get8bits(&mut self) -> Option<u32> {
        if self.bit_count < 8 {
            if self.pos >= self.data.len() {
                return None;
            }
            let reload = self.data[self.pos] as u32;
            self.pos += 1;
            self.bit_buffer |= reload << self.bit_count;
            self.bit_count += 8;
        }
        let out = self.bit_buffer & 0xFF;
        self.bit_buffer >>= 8;
        self.bit_count -= 8;
        Some(out)
    }
}

#[derive(Clone, Copy, Debug)]
enum InsertPoint {
    After,
    Before,
}

#[derive(Clone, Copy, Debug, Default)]
struct TreeItem {
    next: usize,
    prev: usize,
    decompressed_value: u32,
    weight: u32,
    parent: Option<usize>,
    child_lo: Option<usize>,
    linked: bool,
}

#[derive(Debug)]
struct HuffmanTree {
    items: Vec<TreeItem>,
    items_used: usize,
    items_by_byte: [Option<usize>; 0x102],
    is_cmp0: bool,
}

impl HuffmanTree {
    fn new() -> Self {
        Self {
            items: vec![TreeItem::default(); HUFF_ITEM_COUNT + 1], // idx 0 is sentinel
            items_used: 0,
            items_by_byte: [None; 0x102],
            is_cmp0: false,
        }
    }

    fn first(&self) -> usize {
        self.items[0].next
    }

    fn last(&self) -> usize {
        self.items[0].prev
    }

    fn link_two_items(&mut self, item1: usize, item2: usize) {
        // Insert item2 after item1.
        let next = self.items[item1].next;
        self.items[item2].next = next;
        self.items[item2].prev = item1;
        self.items[item2].linked = true;

        self.items[next].prev = item2;
        self.items[item1].next = item2;

        // If list was empty, update sentinel.prev as well.
        if self.items[0].prev == item1 && item1 == 0 {
            self.items[0].prev = item2;
        }
        // If inserting after the last element, update sentinel.prev.
        if next == 0 {
            self.items[0].prev = item2;
        }
    }

    fn insert_before(&mut self, insert_point: usize, new_item: usize) {
        // Insert new_item before insert_point.
        let prev = self.items[insert_point].prev;
        self.items[new_item].next = insert_point;
        self.items[new_item].prev = prev;
        self.items[new_item].linked = true;

        self.items[prev].next = new_item;
        self.items[insert_point].prev = new_item;

        // If inserting before first (i.e. before sentinel.next), update sentinel.next.
        if insert_point == self.items[0].next {
            self.items[0].next = new_item;
        }
        // If insert_point is sentinel (0), then we're inserting at end.
        if insert_point == 0 {
            self.items[0].prev = new_item;
            if self.items[0].next == 0 {
                self.items[0].next = new_item;
            }
        }
    }

    fn remove_item(&mut self, idx: usize) {
        if !self.items[idx].linked {
            return;
        }
        let next = self.items[idx].next;
        let prev = self.items[idx].prev;
        self.items[prev].next = next;
        self.items[next].prev = prev;

        // Fix sentinel pointers if needed
        if self.items[0].next == idx {
            self.items[0].next = next;
        }
        if self.items[0].prev == idx {
            self.items[0].prev = prev;
        }

        self.items[idx].next = 0;
        self.items[idx].prev = 0;
        self.items[idx].linked = false;
    }

    fn insert_item(&mut self, idx: usize, where_: InsertPoint, insert_point: Option<usize>) {
        self.remove_item(idx);
        let ip = insert_point.unwrap_or(0);
        match where_ {
            InsertPoint::After => self.link_two_items(ip, idx),
            InsertPoint::Before => self.insert_before(ip, idx),
        }
    }

    fn create_new_item(
        &mut self,
        decompressed_value: u32,
        weight: u32,
        where_: InsertPoint,
    ) -> Option<usize> {
        if self.items_used >= HUFF_ITEM_COUNT {
            return None;
        }
        self.items_used += 1;
        let idx = self.items_used;
        self.items[idx] = TreeItem {
            decompressed_value,
            weight,
            parent: None,
            child_lo: None,
            ..TreeItem::default()
        };
        self.insert_item(idx, where_, None);
        Some(idx)
    }

    fn find_higher_or_equal_item(&self, mut idx: usize, weight: u32) -> usize {
        while idx != 0 {
            if self.items[idx].weight >= weight {
                return idx;
            }
            idx = self.items[idx].prev;
        }
        0
    }

    fn fixup_item_pos_by_weight(&mut self, new_item: usize, mut max_weight: u32) -> u32 {
        if self.items[new_item].weight < max_weight {
            let higher = self.find_higher_or_equal_item(self.last(), self.items[new_item].weight);
            self.remove_item(new_item);
            self.link_two_items(higher, new_item);
        } else {
            max_weight = self.items[new_item].weight;
        }
        max_weight
    }

    fn build_tree(&mut self, compression_type: u32) -> bool {
        self.items_by_byte = [None; 0x102];
        let mut max_weight = 0u32;

        if (compression_type & 0x0F) > 0x08 {
            return false;
        }
        let table = &WEIGHT_TABLES[(compression_type & 0x0F) as usize];

        // Build initial list
        for (i, entry) in table.iter().enumerate().take(0x100usize) {
            let w = *entry as u32;
            if w != 0 {
                let idx = match self.create_new_item(i as u32, w, InsertPoint::After) {
                    Some(v) => v,
                    None => return false,
                };
                self.items_by_byte[i] = Some(idx);
                max_weight = self.fixup_item_pos_by_weight(idx, max_weight);
            }
        }

        // Termination entries at end
        if let Some(i100) = self.create_new_item(0x100, 1, InsertPoint::Before) {
            self.items_by_byte[0x100] = Some(i100);
        } else {
            return false;
        }
        if let Some(i101) = self.create_new_item(0x101, 1, InsertPoint::Before) {
            self.items_by_byte[0x101] = Some(i101);
        } else {
            return false;
        }

        // Build the Huffman tree from the lowest weights
        let mut child_lo = self.last();
        while child_lo != 0 {
            let child_hi = self.items[child_lo].prev;
            if child_hi == 0 {
                break;
            }
            let weight_sum = self.items[child_hi].weight + self.items[child_lo].weight;
            let parent = match self.create_new_item(0, weight_sum, InsertPoint::After) {
                Some(v) => v,
                None => return false,
            };
            self.items[child_lo].parent = Some(parent);
            self.items[child_hi].parent = Some(parent);
            self.items[parent].child_lo = Some(child_lo);

            max_weight = self.fixup_item_pos_by_weight(parent, max_weight);
            child_lo = self.items[child_hi].prev;
        }

        true
    }

    fn inc_weights_and_rebalance(&mut self, mut item: usize) {
        while item != 0 {
            self.items[item].weight += 1;

            let higher =
                self.find_higher_or_equal_item(self.items[item].prev, self.items[item].weight);
            let child_hi = self.items[higher].next;

            if child_hi != item {
                // Move child_hi to the RIGHT of item
                self.remove_item(child_hi);
                self.link_two_items(item, child_hi);

                // Move item after the higher-weight item
                self.remove_item(item);
                self.link_two_items(higher, item);

                // Rebalance parents
                let parent_of_child_hi = self.items[child_hi].parent;
                if let Some(pch) = parent_of_child_hi {
                    let child_lo = self.items[pch].child_lo;

                    let parent_of_item = self.items[item].parent;
                    if let Some(p) = parent_of_item {
                        if self.items[p].child_lo == Some(item) {
                            self.items[p].child_lo = Some(child_hi);
                        }
                    }

                    if child_lo == Some(child_hi) {
                        self.items[pch].child_lo = Some(item);
                    }

                    // Swap parents.
                    let p_item = self.items[item].parent;
                    self.items[item].parent = self.items[child_hi].parent;
                    self.items[child_hi].parent = p_item;
                }
            }

            match self.items[item].parent {
                Some(p) => item = p,
                None => break,
            }
        }
    }

    fn insert_new_branch_and_rebalance(&mut self, value1: u32, value2: u32) -> bool {
        let last_item = self.last();
        if last_item == 0 {
            return false;
        }

        let child_hi =
            match self.create_new_item(value1, self.items[last_item].weight, InsertPoint::Before) {
                Some(v) => v,
                None => return false,
            };
        self.items[child_hi].parent = Some(last_item);
        self.items_by_byte[value1 as usize] = Some(child_hi);

        let child_lo = match self.create_new_item(value2, 0, InsertPoint::Before) {
            Some(v) => v,
            None => return false,
        };
        self.items[child_lo].parent = Some(last_item);
        self.items[last_item].child_lo = Some(child_lo);
        self.items_by_byte[value2 as usize] = Some(child_lo);

        self.inc_weights_and_rebalance(child_lo);
        true
    }

    fn decode_one_byte(&self, input: &mut InputStream<'_>) -> u32 {
        let mut item = self.first();
        while let Some(child) = self.items[item].child_lo {
            item = match input.get1bit() {
                Some(0) => child,
                Some(_) => self.items[child].prev,
                None => return HUFF_DECOMPRESS_ERROR,
            };
        }
        self.items[item].decompressed_value
    }

    fn decompress_stream(&mut self, input: &[u8], limit: usize) -> Option<Vec<u8>> {
        let mut is = InputStream::new(input);

        let compression_type = is.get8bits()?;
        self.is_cmp0 = compression_type == 0;
        if !self.build_tree(compression_type) {
            return None;
        }

        let mut out = Vec::new();
        loop {
            let mut value = self.decode_one_byte(&mut is);
            if value == 0x100 {
                break;
            }
            if value == HUFF_DECOMPRESS_ERROR {
                return None;
            }

            if value == 0x101 {
                value = is.get8bits()?;
                // An escape may introduce only a symbol absent from the tree.
                if self.items_by_byte[value as usize].is_some() {
                    return None;
                }
                let last_val = self.items[self.last()].decompressed_value;
                if !self.insert_new_branch_and_rebalance(last_val, value) {
                    return None;
                }
                if !self.is_cmp0 {
                    if let Some(idx) = self.items_by_byte[value as usize] {
                        self.inc_weights_and_rebalance(idx);
                    }
                }
            }

            if out.len() == limit {
                return None;
            }
            out.push(value as u8);
            if self.is_cmp0 {
                if let Some(idx) = self.items_by_byte[value as usize] {
                    self.inc_weights_and_rebalance(idx);
                }
            }
        }

        Some(out)
    }
}

pub(super) fn decode(input: &[u8], limit: usize) -> Result<Vec<u8>, Error> {
    HuffmanTree::new()
        .decompress_stream(input, limit)
        .ok_or(Error::InvalidArchive("invalid or oversized Huffman stream"))
}
