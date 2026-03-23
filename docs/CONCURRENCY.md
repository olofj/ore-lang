# Ore — Concurrency Model

> **⚠ Most of this document is ASPIRATIONAL.** The bootstrap compiler
> supports basic concurrency: `spawn()`, `channel()`, `send()`/`recv()`,
> `sleep()`, and `thread_join_all()` using OS threads. The three-layer model
> (pipelines, channels, cells), `each|` parallel pipelines, `select`,
> structured concurrency, and green threads described below are **not yet
> implemented**. See [IMPLEMENTATION.md](IMPLEMENTATION.md) for what works
> today.

## Why No Locks?

Locks, mutexes, semaphores, and memory barriers exist because of one thing:
**shared mutable state**. Two threads reading the same mutable memory need
coordination to avoid seeing torn writes, stale caches, or half-updated
structures.

Every concurrency bug I generate comes from one of:
1. Forgetting to acquire a lock
2. Acquiring locks in the wrong order (deadlock)
3. Holding a lock too long (contention)
4. Sharing state I didn't realize was shared

The solution isn't better lock syntax. It's **eliminating the need for
locks by eliminating shared mutable state as the default.**

## The Three Layers

### Layer 1: Pipelines (90% of concurrent code)

Pipelines don't share state. Data flows through them. Each step receives
a value, produces a value. There's nothing to lock.

```
results := urls
  | each| fetchPage       -- concurrent: each URL processed independently
  | filter .status == 200 -- sequential: operates on collected results
  | map .body             -- sequential
```

`each|` fans out work to a thread pool. Each work item is independent.
Results are collected. No shared state exists, so no coordination is
needed. The runtime handles work-stealing and scheduling.

This covers:
- Parallel map/filter/reduce
- Concurrent I/O (HTTP requests, file reads, DB queries)
- Fan-out/fan-in processing

No locks. No async/await. No channels. Just `each|`.

### Layer 2: Channels (9% of concurrent code)

When concurrent tasks need to communicate, they use channels. Channels
are typed, bounded, and the only way to send data between concurrent
scopes.

```
-- Producer/consumer
ch := channel Int 100    -- buffered channel of Ints, capacity 100

-- Producer pipeline
spawn
  0..1000 | each n -> ch.send n
  ch.close

-- Consumer pipeline
ch | filter > 500 | each print

-- Multiple producers, multiple consumers: just works
-- Channels are multi-producer, multi-consumer by default

-- Select across channels
select
  msg from inbox   -> handleMessage msg
  tick from timer  -> updateState
  _ after 5.seconds -> timeout
```

Channels enforce that **data is transferred, not shared**. When you send
a value into a channel, you give it up. The runtime can enforce this
because the default types are immutable — sending an immutable value
through a channel is just passing a reference. No copy, no ownership
transfer ceremony.

For mutable values: the compiler prevents sending a `mut` reference
through a channel. You must either:
- Send an immutable snapshot (`.freeze` or implicit copy)
- Transfer ownership (value becomes inaccessible in sender's scope)

This is simpler than Rust's `Send`/`Sync` traits because there's only
one mutable reference type and immutable is the default.

### Layer 3: Cells (1% of concurrent code)

Rarely, you genuinely need shared mutable state: a cache, a counter,
a configuration that updates at runtime. For this, we have **cells** —
inspired by Clojure's atoms.

```
-- A cell holds a value that can be atomically updated
counter := cell 0

-- Atomic update: takes a function, applies it, retries on conflict
counter.update (+ 1)

-- Read current value
current := counter.read

-- Compare-and-swap for complex updates
cache := cell Map[Str, Response]()

cache.update map ->
  map.set key response

-- Or with a conditional
cache.cas old ->
  old.set key response    -- only applies if nobody else modified since read
```

Cells provide:
- **Atomic reads**: always see a consistent value
- **Atomic updates**: function applied atomically (retry on contention)
- **No deadlocks**: there's no lock to hold, so ordering is irrelevant
- **No forgotten unlocks**: nothing to unlock

Under the hood, cells use compare-and-swap (CAS) operations. The runtime
handles the retry loop. For high-contention scenarios, the runtime can
switch to a striped lock internally — but this is an implementation
detail, not a user concern.

What cells **don't** allow:
- Holding a mutable reference to the inner value (no `lock()` that returns
  a guard)
- Blocking on an update
- Composing multiple cell updates atomically (use channels for that)

The last point is intentional. If you need to atomically update two things,
that's a coordination problem, and channels are the right tool.

## What About Shared Immutable State?

It just works. Immutable values can be freely shared across threads with
no coordination. This is the payoff of immutable-by-default:

```
config := loadConfig "app.toml"   -- immutable
db := openPool "postgres://..."   -- connection pool (internally synchronized)

-- Every handler can read config and use db with no locks
server 8080 with config, db
  .get "/status" -> { status: "ok", version: config.version }
  .get "/users" -> db.query User "SELECT * FROM users"
```

`config` is immutable — every handler reads it freely. `db` is a
connection pool — its internal synchronization is encapsulated. The user
never thinks about thread safety because the type system prevents unsafe
sharing.

## Memory Model

The language has a simple memory model:

1. **Immutable values**: freely shared, no barriers needed. The GC
   handles memory reclamation. Reference counting for strings means
   predictable cleanup without GC involvement for the most common type.

2. **Mutable values** (`mut`): confined to their declaring scope. Cannot
   be sent through channels. Cannot be captured by spawned tasks. This is
   enforced at compile time.

3. **Cells**: atomic shared state. All reads/writes go through atomic
   operations. The runtime inserts appropriate memory barriers.

4. **Channels**: data transfer between scopes. The runtime handles
   synchronization.

There is **no `unsafe` block for memory**. If you need lock-free data
structures or custom synchronization, you write it as a native extension
(FFI to C/Rust). This is a deliberate limitation — the language is for
application code, not systems code.

## Comparison: Same Problem In Different Languages

**Problem**: Count HTTP status codes from 1000 URLs concurrently.

### Go
```go
var mu sync.Mutex
counts := make(map[int]int)
var wg sync.WaitGroup

for _, url := range urls {
    wg.Add(1)
    go func(u string) {
        defer wg.Done()
        resp, err := http.Get(u)
        if err != nil {
            return
        }
        mu.Lock()
        counts[resp.StatusCode]++
        mu.Unlock()
    }(url)
}
wg.Wait()
```

13 lines. Manual mutex, manual WaitGroup, manual goroutine management,
manual error ignoring, closure variable capture gotcha.

### Rust
```rust
let counts = Arc::new(Mutex::new(HashMap::new()));
let mut handles = vec![];

for url in &urls {
    let counts = Arc::clone(&counts);
    let url = url.clone();
    handles.push(tokio::spawn(async move {
        if let Ok(resp) = reqwest::get(&url).await {
            let mut map = counts.lock().unwrap();
            *map.entry(resp.status().as_u16()).or_insert(0) += 1;
        }
    }));
}
futures::future::join_all(handles).await;
```

13 lines. Arc, Mutex, clone dance, async move, unwrap on lock, manual
handle collection and join.

### This language
```
counts := urls
  | each| http.get
  | filterOk
  | countBy .status
```

4 lines. No locks. No mutex. No Arc. No WaitGroup. No spawn. No join.
The pipeline expresses the algorithm. The runtime handles the
concurrency. `each|` parallelizes the HTTP calls. `filterOk` drops
failures. `countBy` aggregates. Done.

## Structured Concurrency

All concurrent work is scoped. When a scope exits, all tasks spawned
within it are joined (waited for) or cancelled.

```
fn processAll items:List[Item] -> List[Result]
  items | each| process
  -- all concurrent work finishes before this function returns

-- Explicit timeout
items | each| process | timeout 30.seconds
-- cancels any still-running tasks after 30 seconds

-- Explicit cancellation
scope := concurrent
  spawn longTask1
  spawn longTask2
  spawn longTask3

-- if any task fails, all others are cancelled
-- if the parent scope exits, all tasks are cancelled
-- no orphaned background work, ever
```

No goroutine leaks. No forgotten futures. No fire-and-forget unless
explicitly requested with `detach` (which requires the programmer to
acknowledge they're opting out of structured concurrency).

## What This Model Cannot Do

Be honest about limitations:

1. **Fine-grained lock-free data structures**: Use FFI. This language
   won't compete with hand-tuned lock-free queues.
2. **Actor systems**: Channels + spawn can approximate actors, but
   this isn't Erlang. No supervision trees, no hot code reload.
3. **GPU compute**: Out of scope. Use a library that wraps CUDA/Metal
   via FFI.
4. **Real-time guarantees**: GC pauses (even short ones) mean this
   isn't suitable for hard real-time systems.

These are all acceptable exclusions for an application-level language.
The target programs — web services, CLI tools, data processors — don't
need any of them.
