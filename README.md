# Bitmap Indexes, Locking and Distributed Operators

Bitmap index structures, a lock manager with multi-granularity locking, and distributed join
operators, in Rust.

**Context.** This is my implementation for **Lab 2** of **Scalable Data Management Systems**
(TU Darmstadt, FG Data Management, 2025/26, Erasmus+ year). The course provided the project
skeleton: the index and locking traits, the network abstraction, the test suite and the CI pipeline.
This repository contains **only the files I implemented**, so it does not build standalone.

## Architecture

### Bitmap indexes

All three indexes answer the same `range_lookup(start, end) -> IndexIterator` question and differ
only in how the bitmaps are laid out:

| Index | Layout | Range query |
|---|---|---|
| Naive | one bitmap per distinct value in a `HashMap<Value, Vec<u8>>` | OR the bitmaps of every value inside the range |
| Range-encoded | `BTreeMap<Value, bitmap>`, each bitmap = "value ≤ key", built by OR-ing backwards | combine the encoded bitmap of the upper bound with the one of the lower bound |
| Decomposed | one bitmap per digit of the value in a configurable base | evaluate the digit constraints and materialize only matching rows |

Rows are numbered by their position in the table scan, so bitmap bit `i` corresponds to row `i`. The
decomposed index derives the number of digit positions from the maximum value, which means its memory
is `digits * base * n/8` bytes.

`IndexIterator` converts a bitmap into ascending `RowID`s by skipping all-zero bytes wholesale and
walking the set bits of the remaining ones, so iteration cost is proportional to the number of
results, not to the table size.

### Lock manager

`BasicLockTable` keeps one `BasicLockState` per `RowID`:

```
readers: Set<TID>   writer: Option<TID>   queue: VecDeque<(TID, LockRequest)>
```

`lock` grants a shared lock when there is no writer, grants exclusive when there are no readers, and
allows a **shared → exclusive upgrade only when the caller is the sole reader**. Anything else is
queued. `release` drops the caller's lock and then drains the queue in FIFO order, granting every
request that is now compatible.

`MultiGranularityLockTree` implements the hierarchical protocol along a path: ancestors take
intention locks (`IR`/`IW`), the last node takes `R`/`W`, and each request is checked against the
conflict matrix of the node it targets. On the first conflict the traversal aborts and the whole
transaction is rolled back (`release(tid)` removes all its locks recursively).

### Distributed operators

`create_repartitioning_distribution` hashes the join key modulo the peer count (handling negative
values), and `create_replication_distribution` sends a record to every peer. The `Exchange` operator
pulls from its child, forwards each record to the peers selected by the distribution function, and
returns it locally when the current node is among the targets — so local data is never delayed behind
the network. When the child is exhausted it closes the send side and falls back to receiving. The
join constructors (symmetric/asymmetric repartitioning, replication, semi-join reduction) wire these
pieces together on top of the course's join-operator builders.

## Limitations

- **Immutable indexes.** Every index is built once by streaming the table and is never updated; any
  change to the table requires a full rebuild.
- **Memory cost.** The naive index stores one full bitmap per distinct value, and the decomposed one
  multiplies that by the number of digits; only the range-encoded variant is compact.
- **No deadlock detection.** Denied requests are queued with no timeout and no wait-die/wound-wait
  ordering, so two conflicting lock orders can deadlock silently.
- **Upgrades are limited.** A shared lock can only be promoted when the transaction is the sole
  reader; otherwise the caller must release and retry.
- **All-or-nothing multi-granularity locking.** A conflict anywhere on the path releases *all* locks
  of that TID, not just the partially acquired path, so the caller has to restart the transaction.
- **Single-threaded exchange.** The operator processes one record per `next()` call and does no
  batching or backpressure; it relies on the channel abstraction being reliable and ordered.
- **Replication copies.** The replication distribution clones the record once per peer.

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
