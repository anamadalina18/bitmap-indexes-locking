# Bitmap Indexes, Locking and Distributed Operators

Bitmap index structures, a lock manager with multi-granularity locking, and distributed join
operators, in Rust.

**Context.** This is my implementation for **Lab 2** of **Scalable Data Management Systems**
(TU Darmstadt, FG Data Management, 2025/26, Erasmus+ year). The course provided the project
skeleton: the index and locking traits, the network abstraction, the test suite and the CI pipeline.
This repository contains **only the files I implemented**, so it does not build standalone.

## What I implemented

**Bitmap indexes (`src/index/`)**

- `NaiveBitmapIndex` — one bitmap per distinct value in a `HashMap<Value, Vec<u8>>`; a range query
  ORs the bitmaps of all values inside the range.
- `RangeEncodedBitmapIndex` — values are kept in a `BTreeMap` and each bitmap encodes
  "value ≤ key" by OR-ing it with the bitmap of the next larger value (computed in reverse order);
  a range query then combines the encoded bitmap of the upper bound with the one of the lower bound
  instead of OR-ing every value inside the range.
- `DecomposedBitmapIndex` — one bitmap per digit of the value in a configurable base (the number of
  digit positions is derived from the maximum value); a range query evaluates the per-digit
  constraints and only materializes the rows that can match.
- `IndexIterator` — turns a bitmap into ascending `RowID`s, skipping all-zero bytes instead of
  testing every bit.

**Locking (`src/locking/`)**

- `BasicLockTable` — per-`RowID` lock state in a `HashMap`: multiple shared readers, one exclusive
  writer, shared→exclusive upgrade only when the caller is the sole reader, a FIFO queue for
  denied requests, and `release` returning the requests that become grantable.
- `MultiGranularityLockTree` — hierarchical locking along a path (IS/IX/S/X style intentions):
  traverses the tree, checks each level's lock mode against the requested one, aborts and releases
  the partially acquired path on the first conflict, and releases recursively.

**Distributed operators (`src/dist_operator/`)**

- Distribution functions: hash repartitioning on the join key (`key mod peers`) and full replication.
- `Exchange` — sends tuples to the peers selected by the distribution function and interleaves
  locally-produced records with messages received from other peers, always draining local data first
  so the pipeline is not stalled by the network.
- Join constructors for symmetric repartitioning, asymmetric repartitioning, replication and
  semi-join reduction, built on the course's join-operator builders.

## Files

| Path | Contents |
|---|---|
| `src/index/naive_bitmap_index.rs` | Bitmap-per-value index and range OR |
| `src/index/range_encoded_bitmap_index.rs` | Prefix-encoded bitmaps |
| `src/index/decomposed_bitmap_index.rs` | Digit-wise bitmap index |
| `src/index/index_iterator.rs` | Bitmap → `RowID` iterator |
| `src/locking/lock_table.rs` | Shared/exclusive lock table with queueing |
| `src/locking/multi_granularity_locking.rs` | Intention-lock tree |
| `src/dist_operator/distribution/distribution_impl.rs` | Hash and replication distributions |
| `src/dist_operator/exchange/exchange_impl.rs` | Exchange operator |
| `src/dist_operator/join/join_impl.rs` | Distributed join constructors |

## Notes

- Index construction streams the table through the table-scan operator, so index creation uses the
  same access path as query execution.
- Correctness was checked against the course's basic test suite.
