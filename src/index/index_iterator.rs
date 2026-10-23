use crate::RowID;

/// An iterator that translates a bitmap into increasing RowIDs (see description of bitmaps in file "src/index/mod.rs")
pub struct IndexIterator {
    // add necessary members here
    bitmap:Vec<u8>,
    idx_vec:usize,
    idx_bit:usize,
}

impl IndexIterator {
    pub fn new(bitmap: Vec<u8>) -> Self {
        Self {
            bitmap,
            idx_vec: 0,
            idx_bit: 0,
        }
    }
}

impl Iterator for IndexIterator {
    type Item = RowID;

    fn next(&mut self) -> Option<Self::Item> {
        while self.idx_vec < self.bitmap.len() {
            if self.bitmap[self.idx_vec] == 0 {
                self.idx_vec += 1;
                self.idx_bit = 0;
                continue;
            }
            let bit = ((1 << self.idx_bit) & self.bitmap[self.idx_vec]) != 0;

            if bit {
                let row_id = RowID(self.idx_vec * 8 + self.idx_bit);
                self.idx_bit = self.idx_bit + 1;
                if self.idx_bit == 8 {
                    self.idx_vec += 1;
                    self.idx_bit = 0;
                }
                return Some(row_id)
            }

            self.idx_bit += 1;
            if self.idx_bit == 8 {
                self.idx_vec += 1;
                self.idx_bit = 0;
            }
        }

        None
    }
}
