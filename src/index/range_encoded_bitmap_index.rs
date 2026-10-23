use crate::Operator;
use crate::Value;
use crate::index::index_iterator::IndexIterator;
use crate::index::shared::*;
use crate::index::{BitmapIndex, RangeEncodedBitmapIndex};
use crate::test_util::{DummyTable, TableScan};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;

impl RangeEncodedBitmapIndex {
    /// Create a range encoded bitmap index for the records in table,
    /// using the column key_column_number.
    pub fn new(table: Arc<DummyTable>, key_column_number: usize) -> RangeEncodedBitmapIndex {
        let mut map = BTreeMap::new();
        let nr_rec = table.get_number_of_records();
        let len = (nr_rec + 7 ) / 8;
        let mut idx_row = 0;
        let mut scan = TableScan::new(table);

        scan.open();
        let mut elem = scan.next();

        while elem != None {
            let record = elem.unwrap();
            let value = record.get(key_column_number).unwrap();

            if (!map.contains_key(value)) {
                map.insert(value.clone(), vec![0u8; len]);
            }

            let bitmap = map.get_mut(value).unwrap();
            let idx_vec = idx_row / 8;
            let entry = bitmap.get_mut(idx_vec).unwrap();
            let idx_bit = idx_row % 8;
            *entry = *entry | ((1 << idx_bit) as u8);

            elem = scan.next();
            idx_row += 1;
        }

        scan.close();

        let mut prev:Option<Vec<u8>> = Option::None;

        for bitmap in map.values_mut().rev() {
            match prev.clone() {
                None => {},
                Some(prev_bitmap) => {
                    for i in 0..len {
                        bitmap[i] |= prev_bitmap[i];
                    }
                }
            }
            prev = Some(bitmap.clone());
        }

        Self {
            number_of_records: nr_rec,
            map,
        }
    }
}

impl BitmapIndex for RangeEncodedBitmapIndex {
    fn range_lookup(&self, start_key: Value, end_key: Value) -> IndexIterator {
        let len = (self.number_of_records + 7 ) / 8;
        let mut bitmap_final= vec![0u8; len];
        let mut start_bool = true;
        let mut end_bool = true;
        let mut start_bitmap= vec![0u8; len];
        let mut end_bitmap= vec![0u8; len];

        for (value, bitmap) in &self.map {
            if *value >= start_key && *value <= end_key {
                if start_bool {
                    start_bool = false;
                    start_bitmap = bitmap.clone();
                }
            }

            if *value > end_key && end_bool {
                end_bool = false;
                end_bitmap = bitmap.clone();
            }
        }

        if start_bool == false && end_bool == false {
            for i in 0..len {
                bitmap_final[i] = start_bitmap[i] & (!end_bitmap[i])
            }
        }

        if start_bool == false && end_bool == true {
            bitmap_final = start_bitmap;
        }

        IndexIterator::new(bitmap_final)
    }
}
