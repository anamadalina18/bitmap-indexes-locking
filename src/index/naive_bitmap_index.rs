use crate::Operator;
use crate::Value;
use crate::index::index_iterator::IndexIterator;
use crate::index::shared::*;
use crate::index::{BitmapIndex, NaiveBitmapIndex};
use crate::test_util::{DummyTable, TableScan};
use std::collections::HashMap;
use std::ptr::hash;
use std::sync::Arc;

impl NaiveBitmapIndex {
    /// Create a naive bitmap index for the records in table,
    /// using the column key_column_number.
    pub fn new(table: Arc<DummyTable>, key_column_number: usize) -> Self {
        let mut hash_map = HashMap::new();
        let nr_rec = table.get_number_of_records();
        let len = (nr_rec + 7 ) / 8;
        let mut idx_row = 0;
        let mut scan = TableScan::new(table);

        scan.open();
        let mut elem = scan.next();

        while elem != None {
            let record = elem.unwrap();
            let value = record.get(key_column_number).unwrap();

            if (!hash_map.contains_key(value)) {
                hash_map.insert(value.clone(), vec![0u8; len]);
            }

            let bitmap = hash_map.get_mut(value).unwrap();
            let idx_vec = idx_row / 8;
            let entry = bitmap.get_mut(idx_vec).unwrap();
            let idx_bit = idx_row % 8;
            *entry = *entry | ((1 << idx_bit) as u8);

            elem = scan.next();
            idx_row += 1;
        }

        scan.close();

        Self {
            number_of_records: nr_rec,
            hash_map,
        }
    }
}

impl BitmapIndex for NaiveBitmapIndex {
    fn range_lookup(&self, start_key: Value, end_key: Value) -> IndexIterator {
        let len = (self.number_of_records + 7 ) / 8;
        let mut bitmap_final= vec![0u8; len];

        for (value, bitmap) in &self.hash_map {
            if *value >= start_key && *value <= end_key {
                for i in 0..len {
                    bitmap_final[i] |= bitmap[i];
                }
            }
        }

        IndexIterator::new(bitmap_final)
    }
}
