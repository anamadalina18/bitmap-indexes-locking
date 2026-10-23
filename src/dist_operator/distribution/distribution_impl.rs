use crate::{Record, Value};

use super::DistributionFn;

/// Creates a hash distribution function
///
/// All keys are of the type [`Value::Int``]
///
/// Hash function:
/// 'receiving node' = 'key' mod 'number of peers'
///
/// `join_key` is the index of the join key in the `record` vector (i.e., which
/// field in `record` is the actual join key). It is NOT the value of the join key.
pub fn create_repartitioning_distribution(join_key: usize) -> DistributionFn {
    Box::new(move |record: &Record, peers| {
        let val = record.get(join_key).unwrap();
        let Value::Int(x) = val else {
            return vec![0]
        };
        let rest = x % (peers as i32);

        if rest < 0 {
            let aux = peers as i32 + rest;
            return vec![ aux as u16 ]
        }

        vec![ rest as u16 ]
    })
}

/// Creates a replication distribution function
///
/// All keys are of the type [`Value::Int``] and are send to every other node
pub fn create_replication_distribution() -> DistributionFn {
    Box::new(move |_: &Record, peers| (0..peers).collect())
}
