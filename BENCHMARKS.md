# Benchmarks

Measurements for the lexical search engine. Every optimization in Phase 3 and later gets compared against these tables, so keep entries reproducible: same dataset, `--release`, same queries.

**How to reproduce:** `cargo run --release -- seed`, then `cargo run --release`.

---

## Baseline 2 (current): dataset v2, varied catalog

- **Date:** 2026-10-17
- **State:** modules split, `SearchResult { product_id, score }`, products loaded from PostgreSQL, top-10 fetch by ID with ranked reorder
- **Build profile:** `cargo run --release` (assumed; the numbers are consistent with release)
- **Dataset:** 100,000 products from `cargo run -- seed`. Generator in `src/seed.rs`: 8 categories, 24 brands, 30 adjectives, variable-length descriptions. Deterministic.
- **Machine:** _(fill in: CPU, RAM, OS)_
- **Runs:** 1 per row (single run, treat as approximate)

### Search algorithms (`compare`): all matches, ranked

| Query    | Matches | Index build | Naive search | Indexed search | Speedup |
|----------|---------|-------------|--------------|----------------|---------|
| `jacket` | 1,343   | 289.8 ms    | 11.41 ms     | 113 µs         | ~100x   |

### End-to-end flow (`inverted_flow`): search, then fetch top 10 from PostgreSQL

| Query   | Matches | Index build | Indexed search | Fetch top 10 (PostgreSQL) |
|---------|---------|-------------|----------------|---------------------------|
| `shoes` | 3,117   | 284.8 ms    | 361.4 µs       | 1.99 ms                   |

Observations:
- The PostgreSQL fetch (~2 ms) costs several times more than the in-memory search (~0.1-0.4 ms). Once real products are returned, I/O dominates, not the index.
- The index build (~285 ms) happens once at startup and is not part of per-request latency.
- The two tables use different queries (`jacket` vs `shoes`), so their match counts and search times are not directly comparable. Use the same query in both next time.

---

## Baseline 1 (history): dataset v1, uniform catalog

Old generator (15 adjectives x 15 nouns, tied scores). Kept for history only; not comparable with Baseline 2.

Release build, query `jacket`, 6,675 matches:

| Index build | Naive search | Indexed search | Speedup |
|-------------|--------------|----------------|---------|
| 244.5 ms    | 10.96 ms     | 394.3 µs       | ~28x    |

Fetching products from PostgreSQL after the search (same dataset, query `jacket`):

| Products fetched | Fetch time |
|------------------|------------|
| all 6,675        | 13.8 ms    |
| top 10           | 1.88 ms    |

Debug build (`cargo run`), same query: index build 1.84 s, naive 201.3 ms, indexed 3.40 ms. Debug is roughly 8-18x slower. Always benchmark with `--release`.

---

## Known issues affecting results

- Indexed search scores every matching product and sorts all of them (no top-K), so cost includes an O(R log R) sort. Phase 3.1 targets this.
- Many products tie on score (BM25 depends only on text length when term counts match), and tie order is not deterministic. Phase 3.1 needs a tie-break (for example by `product_id`).
- Match counts agree between naive and indexed search; the sets of IDs have not yet been compared.

## TODO for the next baseline

- [ ] Use the same query in `compare` and `inverted_flow`
- [ ] Repeat each measurement several times and record min / median / max
- [ ] Add a rare query, a broad query, a multi-term query (`wireless keyboard`), and a zero-match query
- [ ] Compare the sets of returned IDs for naive vs indexed
- [ ] Record peak memory and index size
- [ ] Separate candidate retrieval time from scoring time
