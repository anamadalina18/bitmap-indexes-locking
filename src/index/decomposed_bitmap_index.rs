use std::cmp::max;
use crate::Operator;
use crate::Value;
use crate::index::index_iterator::IndexIterator;
use crate::index::shared::*;
use crate::index::{BitmapIndex, DecomposedBitmapIndex};
use crate::test_util::{DummyTable, TableScan};
use std::sync::Arc;

impl DecomposedBitmapIndex {
    /// Create a decomposed bitmap index in the specified base for the records in table,
    /// using the column key_column_number. The maximum value (i.e., the one with the longest
    /// representation in terms of digits) in that column is provided here as max_val.
    /// Values in the key column can be expected to always be Value::UInt.,
    /// base is always greater than 1.
    pub fn new(
        table: Arc<DummyTable>,
        key_column_number: usize,
        base: usize,
        max_val: usize,
    ) -> Self {
        let number_of_records = table.get_number_of_records();
        let len = (number_of_records + 7) / 8;
        let mut total_digits = 0;
        let mut max_val_cpy = max_val.clone();

        while max_val_cpy != 0 {
            total_digits += 1;
            max_val_cpy /= base;
        }

        let mut vecs: Vec<Vec<Vec<u8>>> = vec![vec![vec![0u8; len]; base]; total_digits];
        let mut scan = TableScan::new(table);
        scan.open();

        let mut elem = scan.next();
        let mut row_id = 0;

        while elem != None {
            if let Value::UInt(mut val) = elem.unwrap().get(key_column_number).unwrap().clone() {
                let mut nr_digits = 0;

                while val != 0 {
                    let digit = val % base;
                    val = val / base;

                    let idx_bit = row_id % 8;
                    vecs[nr_digits][digit][row_id / 8] = vecs[nr_digits][digit][row_id / 8] | (1 << idx_bit);

                    nr_digits += 1;
                }

                while nr_digits < total_digits {
                    let idx_bit = row_id % 8;
                    vecs[nr_digits][0][row_id / 8] = vecs[nr_digits][0][row_id / 8] | (1 << idx_bit);

                    nr_digits += 1;
                }
            }

            elem = scan.next();
            row_id += 1;
        }

        scan.close();

        Self {
            number_of_records,
            total_digits,
            vecs
        }
    }

    /// This is only used in the benchmark! -> Not relevant for basic and advanced tests.
    /// The return value of this function is used as base for the DecomposedBitmapIndex in the
    /// benchmark.
    ///
    /// To achieve the best performance, you may try to optimize the return value or introduce a
    /// dynamic calculation based on the three input parameters.
    pub fn determine_optimal_base(
        _table: Arc<DummyTable>,
        _key_column_number: usize,
        _max_val: usize,
    ) -> usize {
        if _max_val < 10000 {
            return 6
        }
        5
    }
}

impl BitmapIndex for DecomposedBitmapIndex {
    /// Same as for the creation of the decomposed bitmap index:
    /// start_key and end_key Values can be expected to always be usize.
    fn range_lookup(&self, start_key: Value, end_key: Value) -> IndexIterator {
        let base = self.vecs[0].len();
        let len = (self.number_of_records + 7) / 8;
        let mut digit_start_key = 0;
        let mut digit_end_key = 0;
        let Value::UInt(mut start_key_val) = start_key else {
            panic!();
        };
        let Value::UInt(mut end_key_val) = end_key else {
            panic!();
        };
        let mut bitmap_final = vec![255u8; len];
        let mut vec_digits_start_key:Vec<usize> = vec![0usize; self.total_digits];
        let mut vec_digits_end_key:Vec<usize> = vec![0usize; self.total_digits];

        for (i, j) in vec_digits_start_key.iter_mut().zip(vec_digits_end_key.iter_mut()) {
            if start_key_val != 0 {
                digit_start_key = start_key_val % base;
                start_key_val /= base;
                *i = digit_start_key;
            }

            if end_key_val != 0 {
                digit_end_key = end_key_val % base;
                end_key_val /= base;
                *j = digit_end_key;
            }
        }

        let mut bitmap_and_start_key = vec![255u8; len];
        let mut bitmap_and_end_key = vec![255u8; len];
        let mut bitmap_start = vec![0u8; len];
        let mut bitmap_end = vec![0u8; len];
        for i in 0..self.total_digits {
            let idx = self.total_digits - i - 1;
            for digit in 0..vec_digits_start_key[idx] {
                for ((bitmap, elem_vec), bitmap_and) in bitmap_start.iter_mut().zip(self.vecs[idx][digit].iter()).zip(bitmap_and_start_key.iter()) {
                    *bitmap |= elem_vec & bitmap_and;
                }
            }

            for digit in vec_digits_end_key[idx] + 1..base {
                for ((bitmap, elem_vec), bitmap_and) in bitmap_end.iter_mut().zip(self.vecs[idx][digit].iter()).zip(bitmap_and_end_key.iter()) {
                    *bitmap |= elem_vec & bitmap_and;
                }
            }

            for ((bitmap_start, elem_vecs), (bitmap_end, elem_vecs_end)) in bitmap_and_start_key.iter_mut().zip(self.vecs[idx][vec_digits_start_key[idx]].iter())
                .zip(bitmap_and_end_key.iter_mut().zip(self.vecs[idx][vec_digits_end_key[idx]].iter())) {
                *bitmap_start &= elem_vecs;
                *bitmap_end &= elem_vecs_end;
            }
        }

        for ((bitmap_f, bitmap_s), bitmap_e) in bitmap_final.iter_mut().zip(bitmap_start.iter()).zip(bitmap_end.iter()) {
            *bitmap_f = !bitmap_s & !bitmap_e;
        }

        if self.number_of_records % 8 != 0 {
            let mut mask: u8 = 255;
            mask >>= 8 - (self.number_of_records % 8);
            bitmap_final[len - 1] &= mask;
        }

        IndexIterator::new(bitmap_final)
    }
}
