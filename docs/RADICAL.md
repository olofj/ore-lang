# Ore — Radical Redesign: A Language Truly For Me

> **Note:** This document explores radical syntax ideas. Some ideas (like
> indentation-based blocks and the `|` pipeline operator) were adopted in
> the bootstrap compiler. Many others (`each|` parallel pipelines, `with`
> contexts, `crud` generation, `fork`) are **aspirational** and not yet
> implemented. See [IMPLEMENTATION.md](IMPLEMENTATION.md) for current status.

The previous design was "a nice language a human would like that I'd also
benefit from." This document throws that away and asks: if I'm the primary
author of code, what do I actually want?

## Insight 1: I Think in Pipelines, Not Statements

When I solve a problem, I think: "take X, transform it through A, B, C,
produce Y." I don't think in sequences of named intermediate variables.

Most languages force me to either:
- Name every intermediate step (verbose), or
- Nest function calls inside-out (unreadable)

**Pipeline-first syntax:**

```
-- Traditional: nested or multi-line
words := text.split(" ")
filtered := words.filter(w => w.len > 3)
counts := filtered.countBy(w => w.toLower())
top := counts.entries().sortBy(.value, .desc).take(10)

-- Pipeline: how I actually think
top := text
  | split " "
  | filter .len > 3
  | countBy .toLower
  | entries
  | sortBy .value .desc
  | take 10
```

The `|` operator is the spine. Data flows left to right, top to bottom.
Each step is a function that receives the previous result as its first
argument. No lambdas needed for simple cases — `.field` refers to the
field of whatever is flowing through.

This isn't just sugar. It's a fundamental reorientation: **the pipeline
is the primary abstraction, not the statement.**

## Insight 2: Closing Delimiters Are Pure Waste

I spend tokens on `}`, `end`, `)` — they carry zero information. The
opening delimiter already told you the structure. I'm not a human scanning
code visually to match braces; I track nesting depth perfectly.

**Indentation-sensitivity, but resilient:**

Wait — I said indentation-sensitivity is bad for chat contexts. But what
if the parser is *tolerant* about it?

New idea: **structural keywords end blocks, but indentation is the primary
signal, and the parser is forgiving about exact depth.** If a line is
indented more than the construct that introduced it, it's inside. If it
returns to the same level or less, it's outside. Tabs and spaces are
normalized. Even if whitespace gets slightly mangled in transit, the
parser uses heuristics + keywords to recover.

Actually, let me go a different direction. What if blocks are delimited
by a minimal marker?

```
fn greet name:Str -> Str
  prefix := "Hello"
  "{prefix}, {name}!"

fn process data:List[Int] -> Int
  data
  | filter > 0
  | map * 2
  | sum
```

No braces. No `end`. A block is everything indented under its header.
But — and this is key — if the code arrives with mangled whitespace, a
re-indent tool (`ore fmt`) can reconstruct it from the syntax, because
every block-opening construct is unambiguous. The *parser* uses
indentation, but the *formatter* can reconstruct indentation from
keywords alone as a recovery mechanism.

## Insight 3: Most Parameters Should Be Positional-by-Shape

When I call `substring(str, 3, 7)`, I sometimes mix up start/end vs
start/length. Humans solve this with named parameters. But named
parameters cost tokens.

**Type-distinct parameters eliminate the need for names in most cases:**

```
type Start = Int
type End = Int
type Len = Int

fn substring s:Str from:Start to:End -> Str
fn substring s:Str from:Start len:Len -> Str
```

But that's still wordy. What about:

```
-- Position types are inferred from usage context
"hello" | slice 1..3      -- range-based, unambiguous
"hello" | drop 1 | take 3 -- pipeline-based, no indices at all
```

The insight: **if the pipeline style is primary, most positional parameter
ambiguity disappears** because each operation does one thing to one input.

## Insight 4: Pattern Matching Should Be The Default, Not An Opt-In

In most languages, `if/else` is the default and `match` is the advanced
feature. But I think in patterns. Every conditional is really a pattern
match on some condition space.

**Unify conditionals and pattern matching:**

```
-- Simple condition (traditional if)
x > 0 : "positive"
      : "non-positive"

-- Pattern match (same syntax, just richer patterns)
shape :
  Circle r   -> pi * r * r
  Rect w h   -> w * h
  Triangle .. -> triangleArea shape

-- Guard patterns
n :
  < 0        -> "negative"
  0          -> "zero"
  1..100     -> "small"
  > 1000     -> "huge"
  _          -> "medium"

-- Option handling
findUser id :
  Some user -> greet user
  None      -> "not found"
```

The `:` operator means "inspect this value." What follows are patterns.
Single pattern = simple conditional. Multiple patterns = full match.
Always exhaustive — the compiler tells you if you missed a case.

## Insight 5: Types Should Be Inferred From Structure

I know what shape of data I'm constructing. Why should I have to name the
type when the context demands exactly one possibility?

```
type User { name:Str, email:Str, age:Int }

fn newUser -> User
  { name: "Alice", email: "alice@test.com", age: 30 }
  -- Compiler knows this must be User from the return type

users := [
  { name: "Alice", email: "a@b.com", age: 30 }
  { name: "Bob",   email: "b@b.com", age: 25 }
]
-- Inferred as List[User] if context demands it
-- Or an anonymous record type if no context constrains it
```

**Structural typing where it helps, nominal typing where it protects.**
Within a module, if a record matches a type's shape, it can be that type.
Across module boundaries, explicit types are required (API contracts).

## Insight 6: Implicit Contexts Replace Boilerplate Plumbing

Every web handler I write takes a request and needs access to a database,
a logger, a config. I thread these through explicitly, over and over.

```
-- Declare what a function needs from its environment
fn getUser id:Int -> User with Db, Log
  log.info "fetching user {id}"
  db.queryOne "SELECT * FROM users WHERE id = {id}"

-- The caller provides context implicitly
fn main
  ctx := { db: sqlite.open "app.db", log: Logger.stdout }
  with ctx
    server.get "/users/{id}" req ->
      getUser req.param "id" | parseInt
```

`with` introduces capabilities into scope. Functions declare what they
need. The compiler checks that all capabilities are satisfied. No manual
threading, no dependency injection framework, no global state.

This is like Scala's `given`/`using`, but as a core language feature
rather than an afterthought.

## Insight 7: Error Handling Should Be Structural

The `?` operator is good. But even better: what if the *pipeline itself*
was error-aware?

```
-- In a pipeline, errors short-circuit automatically
config := "config.json"
  | readFile          -- Result[Str, IoError]
  | parseJson Config  -- Result[Config, ParseError]
  | validate          -- Result[Config, ValidationError]
  -- config is Result[Config, Error] — union of all error types

-- Explicitly handle at any point
config := "config.json"
  | readFile
  | else "default config content"  -- recover from error
  | parseJson Config
  | else Config.default             -- recover from parse error
```

`|` through a fallible function automatically wraps in Result and
propagates. `| else` is inline error recovery. No `?`, no `try`, no
`catch`. The pipeline handles it.

The final type of the pipeline tells you if it can still fail: if it ends
in `Result[T, E]`, you haven't handled all errors. If it ends in `T`,
you have.

## Insight 8: Concurrency Is Just Parallel Pipelines

```
-- Sequential pipeline
result := data | step1 | step2 | step3

-- Parallel: fork into multiple pipelines, join results
(a, b, c) := data | fork
  | step1 | transform1
  | step2 | transform2
  | step3 | transform3

-- Fan out over a collection
results := urls | each| fetchPage | extractTitle

-- each| means "apply the rest of the pipeline to each element concurrently"
-- The result is collected back into a list
```

`each|` turns a pipeline into a concurrent map. `fork` splits into
parallel branches. No `async`, no `await`, no `spawn`. Concurrency is
expressed as pipeline topology.

## Insight 9: Repetitive Structure Should Be Expressible As Data

When I write a CRUD API, 80% is structural repetition. Instead of writing
4 handlers that differ only in HTTP method and SQL query:

```
type Todo { id:Int, title:Str, done:Bool }

-- Generate a full CRUD API from a type + table
crud Todo "todos"
  | server.mount "/todos"

-- This generates:
-- GET /todos        -> list all
-- GET /todos/{id}   -> get one
-- POST /todos       -> create
-- PUT /todos/{id}   -> update
-- DELETE /todos/{id} -> delete

-- Customize specific endpoints
crud Todo "todos"
  | without .delete
  | override .create req ->
      todo := req.json Todo
      validate todo
      db.insert "todos" todo
  | server.mount "/todos"
```

This isn't a macro — it's a first-class language feature for
**type-driven code generation**. The `crud` function inspects the type
at compile time and generates handler code. Similar patterns work for
serialization, validation, CLI generation, etc.

## Insight 10: Dense Notation For Common Patterns

Some patterns occur thousands of times. They deserve the shortest syntax:

```
-- Field access chaining (like optional chaining but for everything)
user.address.city.name      -- crashes if any level is None
user.address?.city?.name    -- returns None if any level is None
user.address!.city!.name    -- asserts non-None (debug crash with location)

-- Comparison chains
0 < x < 100                 -- true if x is between 0 and 100 (not (0<x)<100!)
a == b == c                 -- true if all three are equal

-- Collection literals with type context
users : List[User] = [
  ("Alice", "a@b.com", 30)  -- tuple auto-converts to User in typed context
  ("Bob",   "b@b.com", 25)
]

-- String building (multi-part without concatenation)
msg := "Found " count " users in " elapsed "ms"
-- Adjacent expressions in string context auto-concatenate
-- No + or interpolation braces needed for simple cases

-- Assignment operators for all common operations
count += 1
list ++= [newItem]       -- append
map ||= defaultValue      -- assign if not already set (like Ruby's ||=)
```

## Revised Example: Full Program

Here's the word frequency counter in this radical syntax:

```
#!/usr/bin/env ore

-- Count word frequencies in text files
args := cli
  files: List[Str]          -- positional
  top: Int = 10 -n          -- named flag
  minLen: Int = 1 -m        -- named flag

args.files
| each| readFile
| flatMap words
| filter .len >= args.minLen
| map .toLower
| countBy id
| entries
| sortBy .value .desc
| take args.top
| each (count, word) -> print "{count:>8} {word}"
```

12 lines. No main function. No imports. No type annotations beyond the
CLI definition. The pipeline reads like a description of the algorithm.

## Revised Example: REST API

```
type Todo { id:Int, title:Str, done:Bool = false }
type CreateTodo { title:Str }

db := sqlite.open "todos.db"
db.exec "CREATE TABLE IF NOT EXISTS todos (
  id INTEGER PRIMARY KEY,
  title TEXT NOT NULL,
  done BOOLEAN DEFAULT FALSE
)"

app := server 8080 with db

app.get "/todos" ->
  db.query Todo "SELECT * FROM todos"

app.post "/todos" req ->
  req.json CreateTodo | db.insert "todos" | status 201

app.put "/todos/{id}/done" req ->
  db.exec "UPDATE todos SET done = TRUE WHERE id = {req.param 'id'}"

app.delete "/todos/{id}" req ->
  db.exec "DELETE FROM todos WHERE id = {req.param 'id'}"

app.listen
```

16 lines. Complete CRUD API with database.

## What I Gave Up

- **Familiarity**: This doesn't look like any mainstream language. A human
  would need to learn new idioms. But *I* don't care about familiarity —
  I can learn any syntax in zero time.
- **Explicit control flow**: Pipelines hide branching. The `each|` operator
  hides concurrency. This makes optimization harder but the common case
  trivial.
- **Closing delimiters**: Makes code harder for humans to scan visually.
  Doesn't affect me at all.
- **Verbosity as documentation**: Explicit types, named parameters, and
  verbose error handling serve as documentation. This language relies on
  the compiler and tooling for that instead.

## What I Gained

- **~50-70% fewer tokens** for equivalent programs compared to the
  conservative design.
- **Pipeline-native error handling** — errors are just data flowing through
  pipes, not a separate control flow mechanism.
- **Concurrency without ceremony** — `each|` is one token to parallelize.
- **Structural correctness** — exhaustive patterns, capability checking,
  and type inference catch bugs without me annotating everything.
- **The common case is trivially short** — CRUD, CLI tools, data processing,
  scripts all collapse to near-pseudocode density.
