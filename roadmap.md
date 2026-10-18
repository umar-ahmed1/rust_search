# Rust Hybrid Product Search Engine --- Development Roadmap

## Project Goal

Build a production-like product search engine in Rust that combines:

-   **Lexical search:** an inverted index with BM25 ranking.
-   **Database-backed product data:** PostgreSQL as the source of truth.
-   **Efficient retrieval:** top-K selection, advanced postings-list
    algorithms, and compact data structures.
-   **An HTTP API:** a service that other applications can query.
-   **Persistent indexing:** on-disk index structures, segments,
    updates, and recovery.
-   **Vector search:** semantic retrieval using text embeddings and
    approximate nearest-neighbor algorithms.
-   **Hybrid ranking:** combine lexical and semantic results.
-   **Production engineering:** testing, benchmarking, observability,
    deployment, and reliability.

The intended sequence is to build a correct lexical engine first, make
it usable as a service, improve storage and performance, and then add
vector search. Keep the project runnable at every stage and use
measurements to guide optimization.

------------------------------------------------------------------------

# Phase 2 --- Replace Hardcoded Products with a Database

**Goal:** use PostgreSQL as the source of truth for product data while
keeping your own inverted index responsible for lexical retrieval.

PostgreSQL is a good choice for this project. It is widely used,
integrates well with Rust, and can also store vector embeddings later
through `pgvector`.

## 2.1 --- Set Up PostgreSQL

-   [x] Run PostgreSQL locally, preferably using Docker.
-   [x] Create a `products` table with fields such as:
    -   `id`
    -   `title`
    -   `description`
    -   `category`
    -   `price`
    -   `in_stock`
    -   `updated_at`
-   [x] Add a primary key and appropriate field constraints.
-   [x] Add database indexes for fields used in filtering or sorting.
-   [ ] Create a realistic seed dataset containing thousands of
    products.
-   [x] Keep the dataset reproducible so benchmarks can be repeated.

## 2.2 --- Connect from Rust

-   [x] Learn the basics of `sqlx` and asynchronous database access.
-   [x] Configure database connections and a connection pool.
-   [ ] Implement product creation, retrieval, updating, and deletion.
-   [ ] Use parameterized queries.
-   [ ] Handle database errors rather than unwrapping every operation.
-   [x] Add database migrations so schema changes are repeatable.
-   [x] Keep credentials and connection strings in configuration or
    environment variables rather than hardcoding them.

## 2.3 --- Separate Application Layers

A possible project layout:

``` text
src/
├── main.rs
├── products/
│   ├── model.rs
│   └── repository.rs
├── indexing/
│   ├── tokenizer.rs
│   └── inverted_index.rs
├── search/
│   ├── bm25.rs
│   └── engine.rs
└── config.rs
```

Introduce modules as the code grows; this exact layout is not mandatory.

Responsibilities:

-   `repository`: fetches and modifies product records.
-   `indexing`: transforms products into searchable structures.
-   `search`: retrieves candidates and ranks them.
-   `main`: wires components together.

Tasks:

-   [ ] Move product data types into an appropriate module.
-   [ ] Keep database access out of the BM25 scoring implementation.
-   [ ] Keep indexing logic separate from database repository logic.
-   [ ] Make the search engine accept the index and return ranked
    product IDs or result references.
-   [ ] Avoid tightly coupling search logic to a particular HTTP
    framework or database driver.

## 2.4 --- Design the Database--Index Relationship

**PostgreSQL stores canonical product data. The inverted index stores
the structures needed to find matching product IDs efficiently.**

-   [ ] Load product records from PostgreSQL to build the index.
-   [ ] Keep the index in memory initially.
-   [ ] Return ranked product IDs from the search engine.
-   [ ] Fetch the corresponding product details by ID.
-   [ ] Ensure product updates and deletions eventually reach the index.
-   [ ] Track the index version or generation so you know which data it
    represents.
-   [ ] Avoid querying PostgreSQL separately for every query term.
-   [ ] Define what happens if a product changes in PostgreSQL while the
    search index is temporarily stale.

**Phase complete when:** the application can restart, load products from
PostgreSQL, build the index, and search products without hardcoded
sample objects.

------------------------------------------------------------------------

# Phase 3 --- Advanced Lexical Retrieval and Data Structures

**Goal:** make retrieval more efficient and learn the algorithms used by
serious search engines.

## 3.1 --- Implement Top-K Retrieval

Instead of sorting every matching document when a user only wants ten
results:

-   [ ] Add a `limit` parameter to the search function.
-   [ ] Implement a bounded heap that retains the best K results.
-   [ ] Sort only the retained results before returning them.
-   [ ] Handle ties and deterministic ordering.
-   [ ] Benchmark the new implementation against full sorting.
-   [ ] Test edge cases such as `K = 0`, `K = 1`, and `K` greater than
    the number of matches.

If `R` documents match a query and you want `K` results, full sorting
costs approximately `O(R log R)`. A bounded heap typically reduces this
to `O(R log K + K log K)`, assuming each candidate score can be computed
independently.

## 3.2 --- Improve Postings-List Retrieval

The current index stores a list of document IDs and term frequencies for
each term.

-   [ ] Make posting lists ordered by document ID.
-   [ ] Experiment with compact representations for document IDs and
    term frequencies.
-   [ ] Implement intersections for queries that require all terms.
-   [ ] Implement unions for queries that match any term.
-   [ ] Add efficient category, price, and availability filtering.
-   [ ] Experiment with skip pointers or block-based skipping.
-   [ ] Compare different query execution strategies on selective and
    broad queries.

A search engine should try to avoid examining candidates that cannot
make the top-K results.

## 3.3 --- Improve Scoring Efficiency

-   [ ] Precompute document frequency for each term.
-   [ ] Avoid recalculating query-invariant information.
-   [ ] Avoid unnecessary `String` allocations during tokenization.
-   [ ] Use compact document IDs and data structures where appropriate.
-   [ ] Measure allocation counts and memory usage.
-   [ ] Compare `HashMap`-based structures with more compact
    alternatives.
-   [ ] Separate the cost of candidate retrieval from the cost of
    scoring.

## 3.4 --- Implement Advanced Retrieval Algorithms

Work through these after the simpler implementation is correct:

-   [ ] **Conjunctive retrieval:** documents must contain all query
    terms.
-   [ ] **Disjunctive retrieval:** documents may contain any query term.
-   [ ] **WAND:** use score bounds to skip candidates that cannot enter
    the top-K results.
-   [ ] **Block-Max WAND:** use block-level score bounds to skip more
    work.
-   [ ] **Field-aware scoring:** title, description, brand, and category
    can have different weights.

WAND and Block-Max WAND are particularly useful learning projects
because they combine ranking, data structures, and algorithmic
performance.

**Phase complete when:** the engine returns top-K results efficiently,
supports filters, and demonstrates measurable gains over the original
implementation.

------------------------------------------------------------------------

# Phase 4 --- Expose the Engine Through an HTTP API

**Goal:** turn the search engine into a service that a frontend or
another application can query.

Recommended starting tools:

-   **Axum:** Rust HTTP routing and request handling.
-   **Tokio:** asynchronous runtime.
-   **Serde:** request and response serialization.

## 4.1 --- Build the First Endpoint

Example request:

``` http
GET /search?q=warm+winter+shoes&limit=10
```

Example response:

``` json
{
  "query": "warm winter shoes",
  "results": [
    {
      "id": 42,
      "title": "Insulated Winter Boots",
      "score": 5.82
    }
  ],
  "total": 1
}
```

The product and score are illustrative.

-   [ ] Create the Axum application.
-   [ ] Define search request and response types.
-   [ ] Connect the endpoint to the existing search engine.
-   [ ] Validate parameters such as `limit`.
-   [ ] Return structured JSON.
-   [ ] Implement sensible HTTP status codes and error responses.
-   [ ] Add a basic endpoint for fetching a product by ID if useful.

## 4.2 --- Make It Behave Like a Service

-   [ ] Load the index at startup rather than rebuilding it for each
    request.
-   [ ] Share the index safely between concurrent requests.
-   [ ] Separate CPU-heavy search work from asynchronous I/O where
    appropriate.
-   [ ] Configure request timeouts and resource limits.
-   [ ] Add health and readiness endpoints.
-   [ ] Handle graceful shutdown.
-   [ ] Ensure database connection pools are reused.
-   [ ] Avoid holding locks across slow I/O operations.

## 4.3 --- Test the API

-   [ ] Unit-test ranking functions.
-   [ ] Integration-test HTTP requests and responses.
-   [ ] Test malformed queries and invalid parameters.
-   [ ] Test concurrent requests.
-   [ ] Test database failures and unavailable index states.
-   [ ] Add API documentation, for example with an OpenAPI-compatible
    approach.
-   [ ] Test the API with `curl` or a small frontend.

**Phase complete when:** you can run the service and search products
from a browser, `curl`, or a frontend.

------------------------------------------------------------------------

# Phase 5 --- Persistent Indexing and Storage-Engine Architecture

**Goal:** make the search index durable and efficient to maintain.

Database persistence and search-index persistence are separate problems.
PostgreSQL persists the product records; this phase makes the search
structures durable and efficient to update.

## 5.1 --- Persist and Reload the Index

-   [ ] Define a versioned on-disk index format.
-   [ ] Serialize the index or write its structures to binary files.
-   [ ] Persist document metadata, term dictionaries, postings, and
    document lengths.
-   [ ] Load the index without re-tokenizing every product.
-   [ ] Validate the index format version.
-   [ ] Detect corrupt or incompatible files and fail safely.
-   [ ] Compare search results before and after saving and loading.
-   [ ] Benchmark loading a persisted index against rebuilding it from
    PostgreSQL.

Start with a straightforward format. The first goal is reliable
persistence, not the perfect binary layout.

## 5.2 --- Build Immutable Index Segments

Rather than modifying one enormous index for every change:

-   [ ] Represent an indexed batch as a segment.
-   [ ] Write new or changed products to a new segment.
-   [ ] Search across multiple segments.
-   [ ] Maintain deletion markers or tombstones.
-   [ ] Merge small segments into larger ones in the background.
-   [ ] Reclaim deleted or obsolete data during merges.
-   [ ] Measure the trade-offs between segment count, query latency, and
    merge cost.

This is a simplified version of an architecture used by modern search
engines.

## 5.3 --- Make Updates Safe

-   [ ] Build new index versions without breaking active queries.
-   [ ] Publish a completed index version atomically.
-   [ ] Keep the previous version available until the new version is
    ready.
-   [ ] Handle crashes during writes and merges.
-   [ ] Make rebuilds resumable or safely repeatable.
-   [ ] Add consistency checks against the product database.
-   [ ] Define how index generations are created, published, and
    retired.

**Phase complete when:** startup is faster than a full rebuild, and
products can be updated while searches continue to work.

------------------------------------------------------------------------

# Phase 6 --- Advanced Performance and Production Reliability

**Goal:** optimize based on evidence rather than guesses.

## 6.1 --- Benchmark the Whole System

-   [ ] Benchmark individual ranking functions.
-   [ ] Benchmark complete search requests.
-   [ ] Measure p50, p95, and p99 latency.
-   [ ] Measure queries per second under different concurrency levels.
-   [ ] Measure resident memory and index size on disk.
-   [ ] Test selective queries and queries that match a large fraction
    of the catalog.
-   [ ] Keep benchmark datasets and query workloads reproducible.
-   [ ] Compare performance before and after each major optimization.

## 6.2 --- Profile and Optimize

-   [ ] Use Rust profiling tools to identify CPU hotspots.
-   [ ] Investigate memory allocation and copying.
-   [ ] Reduce unnecessary string storage and cloning.
-   [ ] Compare data layouts for cache locality.
-   [ ] Evaluate compression of document IDs and term frequencies.
-   [ ] Compare heap-based top-K with WAND-style retrieval.
-   [ ] Re-run benchmarks after each change.
-   [ ] Document the measured improvement and its trade-offs.

## 6.3 --- Add Observability

-   [ ] Add structured logs for errors and important operations.
-   [ ] Record metrics for request volume, latency, and failure rate.
-   [ ] Track index size, generation, and rebuild status.
-   [ ] Track database connection-pool utilization.
-   [ ] Track search result counts and query execution time.
-   [ ] Add health checks and actionable error messages.
-   [ ] Avoid logging sensitive data unnecessarily.

## 6.4 --- Test Failure Scenarios

-   [ ] PostgreSQL becomes unavailable.
-   [ ] The index file is missing or corrupt.
-   [ ] An indexing job fails halfway through.
-   [ ] A request is unusually expensive.
-   [ ] Many searches arrive simultaneously.
-   [ ] The process shuts down during a rebuild.
-   [ ] The application starts with a stale index.
-   [ ] The service recovers without publishing a partially built index.

**Phase complete when:** you can explain bottlenecks using measurements
and demonstrate recovery from realistic failures.

------------------------------------------------------------------------

# Phase 7 --- Build a Vector Search Engine

**Goal:** add semantic retrieval so queries can find relevant products
even when the wording differs.

For example, a lexical search for "shoes that keep me warm" may struggle
with a product described as "insulated winter boots." A text embedding
model can represent the meaning of the query and product in a shared
vector space.

## 7.1 --- Learn and Generate Embeddings

-   [ ] Understand embeddings as vectors representing text or other
    items.
-   [ ] Choose a pretrained text embedding model.
-   [ ] Generate embeddings for product titles and descriptions.
-   [ ] Store each embedding alongside a stable product ID.
-   [ ] Generate an embedding for each incoming query.
-   [ ] Ensure query and product embeddings use compatible models and
    dimensions.
-   [ ] Version the model and record which model produced each vector.
-   [ ] Decide how product title and description text will be combined
    for embedding.

You do not need to train a model initially. A pretrained model lets you
focus on search-engine architecture.

## 7.2 --- Implement Exact Vector Search

Before building an advanced index, implement the simplest correct
version.

-   [ ] Store vectors in memory.
-   [ ] Implement cosine similarity or dot-product scoring, consistent
    with the model's intended use.
-   [ ] Compare the query vector against every product vector.
-   [ ] Return the top-K most similar products.
-   [ ] Test semantic examples where query and product keywords differ.
-   [ ] Measure latency and memory as the number of vectors grows.
-   [ ] Use exact search as the ground-truth baseline for evaluating
    faster approximate methods.

## 7.3 --- Build an Approximate Nearest-Neighbor Index

A brute-force scan becomes expensive for large vector collections. Next,
learn approximate nearest-neighbor (ANN) retrieval.

-   [ ] Start with the exact-search baseline and understand its
    computational cost.
-   [ ] Learn graph-based retrieval, especially **HNSW**.
-   [ ] Experiment with graph construction, neighbor selection, and
    search breadth.
-   [ ] Compare approximate results against exact search.
-   [ ] Measure recall@K, latency, memory, and index-build time.
-   [ ] Explore quantization after understanding the basic graph index.
-   [ ] Document the trade-off between speed, memory, and recall.

HNSW is a useful advanced data-structures project. You can implement a
simplified version yourself and then compare it with a mature library.

## 7.4 --- Integrate Vector Storage and Updates

-   [ ] Store vectors in a dedicated vector index or a database
    extension.
-   [ ] Map every vector to a stable product ID.
-   [ ] Handle changed and deleted products.
-   [ ] Version the embedding model and vector index.
-   [ ] Re-embed products when changing models.
-   [ ] Test consistency between product records and vector records.
-   [ ] Measure the cost of initial embedding generation and ongoing
    updates.

A practical first implementation can use PostgreSQL with `pgvector`. For
deeper systems learning, build your own ANN index as a separate
experiment and compare the two approaches.

**Phase complete when:** semantic search retrieves relevant products
even when the query and product descriptions use different wording.

------------------------------------------------------------------------

# Phase 8 --- Combine BM25 and Vector Search into Hybrid Search

**Goal:** combine lexical and semantic retrieval because they solve
different problems.

-   **BM25:** exact terms, product names, brands, model numbers, and
    rare keywords.
-   **Vector search:** semantic similarity, synonyms, paraphrases, and
    intent.
-   **Filters:** hard requirements such as price, category, or
    availability.
-   **Reranking:** a later stage that improves the order of a smaller
    candidate set.

## 8.1 --- Build Two Independent Retrieval Paths

``` text
                   Search query
                    /         \
                   /           \
             BM25 search     Vector search
                  |               |
             Top 100 IDs      Top 100 IDs
                  \               /
                   \             /
                    Candidate union
                           |
                     Hybrid ranking
                           |
                         Top 10
```

-   [ ] Query the BM25 engine.
-   [ ] Query the vector engine.
-   [ ] Retrieve a configurable number of candidates from each.
-   [ ] Combine product IDs and remove duplicates.
-   [ ] Preserve individual scores and ranks for debugging.
-   [ ] Apply hard filters consistently to the retrieval and final
    result stages.
-   [ ] Ensure deleted or unavailable products do not appear as valid
    results.

The candidate counts above are starting examples, not universal
defaults.

## 8.2 --- Combine Rankings with Reciprocal Rank Fusion

BM25 scores and vector similarities are not naturally on the same scale.
Avoid simply adding raw values and assuming the result is meaningful.

Start with **Reciprocal Rank Fusion (RRF)**:

\[ `\operatorname{RRF}`{=tex}(d)= `\sum`{=tex}\_{r
`\in `{=tex}`\text{rankings}`{=tex}}
`\frac{1}{k+\operatorname{rank}_r(d)}`{=tex} \]

Each ranking contributes based on the document's position in that
ranking. The constant (k) controls how quickly the contribution
declines; a value around 60 is a common starting point.

-   [ ] Implement RRF.
-   [ ] Compare BM25-only, vector-only, and hybrid results.
-   [ ] Evaluate all approaches on the same query set.
-   [ ] Experiment with candidate counts and fusion parameters.
-   [ ] Inspect cases where one retrieval method finds relevant products
    that the other misses.
-   [ ] Log component ranks and scores during development.

## 8.3 --- Add Query-Specific Behavior

-   [ ] Test exact product-name and SKU searches.
-   [ ] Test descriptive queries such as "shoes that keep me warm."
-   [ ] Test brand names and rare terms.
-   [ ] Test queries with hard price or availability constraints.
-   [ ] Compare results for ambiguous queries.
-   [ ] Decide when exact lexical matches should receive extra weight.
-   [ ] Consider a reranking stage for a small candidate set.
-   [ ] Evaluate whether hybrid search improves relevance rather than
    assuming it does.

After RRF, experiment with score normalization and weighted
combinations. A learned reranker is a later option if you have useful
relevance labels.

**Phase complete when:** you can demonstrate, with an evaluation set,
how hybrid search compares with either approach alone.

------------------------------------------------------------------------

# Phase 9 --- Production Hardening and Deployment

**Goal:** make the project credible as a service that can be deployed
and operated.

## 9.1 --- Packaging and Configuration

-   [ ] Containerize the service and database.
-   [ ] Add environment-based configuration.
-   [ ] Keep secrets out of source control.
-   [ ] Create reproducible database migrations and deployments.
-   [ ] Add CI checks for tests, formatting, linting, and dependency
    issues.

## 9.2 --- API and Resource Protection

-   [ ] Add request-size limits.
-   [ ] Configure timeouts and resource limits.
-   [ ] Add rate limiting if appropriate for the deployment.
-   [ ] Validate user-controlled filters and limits.
-   [ ] Ensure expensive queries cannot consume unbounded resources.
-   [ ] Return consistent error responses.

## 9.3 --- Index and Data Operations

-   [ ] Implement a safe index rebuild process.
-   [ ] Support rollback to a previous valid index version.
-   [ ] Keep product, lexical-index, and embedding versions consistent.
-   [ ] Define how indexing failures are reported and retried.
-   [ ] Test backup, restore, and recovery procedures.
-   [ ] Document how to inspect index health and rebuild it.

## 9.4 --- Deployment and Load Testing

-   [ ] Deploy the service to a suitable environment.
-   [ ] Load-test the deployed API.
-   [ ] Monitor latency, errors, memory, and CPU.
-   [ ] Verify that restarts and deployments do not publish incomplete
    indexes.
-   [ ] Document architecture, benchmarks, trade-offs, and known
    limitations.
-   [ ] Add replication or sharding only when you have a measured reason
    to do so.

**Phase complete when:** the service can be deployed reproducibly,
monitored, tested under load, and recovered after realistic failures.

------------------------------------------------------------------------

# Recommended Technology Stack

  Component              Starting choice
  ---------------------- -------------------------------------------------------
  Language               Rust
  Product database       PostgreSQL
  Rust database access   `sqlx`
  HTTP API               Axum + Tokio
  Serialization          Serde
  Lexical index          Your own inverted index
  Lexical ranking        BM25
  Top-K                  Binary heap, then WAND / Block-Max WAND
  Index persistence      Your own versioned segment format
  Embeddings             A pretrained text embedding model
  Vector baseline        Exact cosine similarity or dot product
  ANN                    Your own simplified HNSW, then compare with a library
  Hybrid ranking         Reciprocal Rank Fusion
  Testing                Rust unit and integration tests + relevance dataset
  Benchmarking           Criterion and load-testing tools
  Deployment             Docker, followed by CI/CD and monitoring

These are starting choices, not requirements to adopt everything at
once.

------------------------------------------------------------------------

# Suggested Project Milestones

Treat each substep as a small milestone. Keep the project runnable,
benchmark changes, and commit each working feature.

1.  Finish BM25 tests and record a performance baseline.
2.  Move product storage to PostgreSQL.
3.  Separate repository, indexing, and search responsibilities.
4.  Implement bounded-heap top-K retrieval.
5.  Improve postings lists, filters, and advanced retrieval algorithms.
6.  Expose the engine through an Axum HTTP API.
7.  Persist and reload the lexical index.
8.  Introduce immutable segments, updates, and safe index publication.
9.  Profile the service and add observability and failure tests.
10. Implement exact vector search.
11. Build or integrate an ANN index and evaluate its recall.
12. Combine lexical and vector candidates using RRF.
13. Evaluate and improve hybrid relevance.
14. Containerize, deploy, load-test, and document the complete system.

## Definition of Success

By the end, the project should have:

-   A Rust search service with a database-backed product catalog.
-   A custom lexical index and BM25 ranking implementation.
-   Efficient top-K retrieval and advanced retrieval experiments.
-   A usable HTTP API.
-   Persistent index storage and safe update workflows.
-   Benchmarks, automated tests, metrics, and recovery procedures.
-   Semantic vector retrieval and approximate nearest-neighbor search.
-   Hybrid lexical/vector ranking evaluated against a relevance dataset.
-   Deployment instructions and clear documentation of architecture and
    trade-offs.

**Recommended sequence:** finish Phase 1 → PostgreSQL → advanced lexical
retrieval → HTTP API → persistent segments → performance and reliability
→ exact vector search → ANN → hybrid ranking → deployment hardening.

This sequence builds on your current work without skipping the
fundamentals or adding distributed-system complexity before you have a
reason to need it.
