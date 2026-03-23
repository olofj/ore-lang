# Ore — Bootstrap Compiler: What Works Today

This document describes what the **bootstrap compiler** (written in Rust, in
`bootstrap/`) actually supports right now. If a feature is listed here, you
can use it and expect it to compile and run.

For aspirational/planned features, see [FEATURES.md](FEATURES.md) (sections
marked `ASPIRATIONAL`).

## Syntax

Ore uses **indentation-based blocks** (Python-like). No braces, no semicolons.
Blocks are delimited by indentation under their header.

```
fn greet name:Str -> Str
  prefix := "Hello"
  "{prefix}, {name}!"
```

Comments use `--` (line comment) and `---` (doc comment).

## Variables

```
-- Immutable binding (default)
x := 42
name := "hello"

-- Mutable binding
mut counter := 0
counter = counter + 1

-- Type annotation
x: Int := 42
```

## Functions

```
fn add a:Int b:Int -> Int
  a + b

-- Return type inferred for private functions
fn double x:Int
  x * 2

-- Default parameters
fn greet name:Str greeting:Str = "Hello"
  "{greeting}, {name}!"

-- Single-expression shorthand
fn square x:Int = x * x
```

## Primitive Types

```
Int       -- 64-bit signed integer
Float     -- 64-bit float
Bool      -- true / false
Str       -- UTF-8 string, immutable, reference-counted
```

## String Operations

String interpolation:
```
name := "world"
print "Hello, {name}!"
print "sum is {a + b}"
```

String methods: `len()`, `contains()`, `trim()`, `split()`, `replace()`,
`starts_with()`, `ends_with()`, `to_upper()`, `to_lower()`, `capitalize()`,
`substr()`, `repeat()`, `pad_left()`, `pad_right()`, `chars()`, `index_of()`,
`slice()`, `count()`, `strip_prefix()`, `strip_suffix()`, `lines()`,
`split_whitespace()`, `ord()`, `chr()`, `to_int()`, `to_float()`.

Triple-quoted strings for multiline:
```
text := """
  This is a multiline string.
  Common indentation is stripped.
"""
```

## Control Flow

```
-- If/else (expression-oriented)
result := if x > 0
  "positive"
else
  "non-positive"

-- If/else-if/else
if condition
  doSomething()
else if other
  doOther()
else
  fallback()

-- While loop
while condition
  doWork()

-- For loop over range
for i in range(0, 10)
  print i

-- For-each over collection
for item in items
  process(item)

-- Loop with break
loop
  data := poll()
  if data.ready
    break
```

## Pattern Matching

```
match value
  0 -> "zero"
  n if n > 100 -> "big: {n}"
  _ -> "other"

-- Destructuring sum types
match shape
  Circle(r) -> 3.14159 * r * r
  Rect(w, h) -> w * h

-- Match on Result/Option
match fetchUser(id)
  Ok(user) -> renderProfile(user)
  Err(e) -> handleError(e)
```

Exhaustive matching is enforced for sum types.

The `:` match syntax also works:
```
shape :
  Circle r   -> pi * r * r
  Rect w h   -> w * h
```

## Types

### Record Types (Product Types)

```
type Point
  x: Float
  y: Float

-- With defaults
type Config
  host: Str = "localhost"
  port: Int = 8080

-- Construction
p := Point(x: 3.0, y: 4.0)
c := Config(port: 3000)
```

### Sum Types (Enums)

```
type Shape
  Circle(radius: Float)
  Rect(width: Float, height: Float)

-- Simple enums
type Color
  Red
  Green
  Blue
```

### Built-in Option and Result

```
type Option[T]
  Some(T)
  None

type Result[T, E]
  Ok(T)
  Err(E)
```

`T?` is sugar for `Option[T]`.

The `?` operator propagates errors from `Result` values.

Optional chaining: `obj?.field`, `obj?.method()`.

## Generics

```
fn first[T] items:List[T] -> Option[T]
  if items.is_empty()
    None
  else
    Some(items.get(0))
```

Generics are implemented via monomorphization (a concrete version is generated
for each type used).

## Traits and Impl Blocks

```
trait Display
  fn display self -> Str

impl Display for Point
  fn display self -> Str
    "({self.x}, {self.y})"
```

## Closures / Lambdas

```
doubled := items.map(x => x * 2)
evens := items.filter(x => x % 2 == 0)
```

Closures capture free variables from the enclosing scope.

## Collections

### List[T]

```
nums := [1, 2, 3, 4, 5]
```

Mutation: `push()`, `pop()`, `insert()`, `remove_at()`, `clear()`, `set()`.

Access: `get()`, `len()`, `is_empty()`, `slice()`, `first()`, `last()`.

Higher-order: `map()`, `filter()`, `reduce()`, `fold()`, `scan()`, `each()`,
`sort()`, `sort_by()`, `reverse()`, `unique()`, `take()`, `skip()`,
`flatten()`, `flat_map()`, `zip()`, `enumerate()`, `sum()`, `product()`,
`min()`, `max()`, `average()`, `any()`, `all()`, `find()`, `contains()`,
`join()`.

List comprehensions:
```
squares := [x * 2 | x in items if x > 5]
```

### Map[Str, V]

Maps are **string-keyed only** in the bootstrap compiler.

```
m := Map()
m.set("key", value)
v := m.get("key")
```

Methods: `set()`, `get()`, `contains()`, `len()`, `remove()`, `keys()`,
`values()`, `entries()`, `merge()`.

## Pipeline Operator

```
result := data
  | filter(x => x > 0)
  | map(x => x * 2)
  | sum()
```

Multi-line pipelines with `|` are supported.

## Concurrency

```
-- Spawn a thread
handle := spawn(myFunction)

-- Spawn with arguments
handle := spawn_with_arg(myFunction, arg1)

-- Join all threads
thread_join_all(handles)

-- Channels
ch := channel()
ch.send(value)
received := ch.recv()

-- Sleep
sleep(1000)  -- milliseconds
```

Note: concurrency uses OS threads, not green threads. `spawn` runs a function
in a new thread. Channels are typed and multi-producer/multi-consumer.

## I/O and System

```
-- Console
print("Hello")
eprint("Error message")
input := readln()

-- File I/O
content := file_read("path.txt")
file_write("path.txt", content)
file_append("path.txt", more)
exists := file_exists("path.txt")
lines := file_read_lines("path.txt")

-- Shell commands
result := exec("ls -la")

-- Command-line arguments
arguments := args()

-- Environment variables
val := env_get("HOME")
env_set("KEY", "VALUE")

-- Exit
exit(0)
```

## Math

Functions: `abs()`, `min()`, `max()`, `sqrt()`, `sin()`, `cos()`, `tan()`,
`log()`, `log10()`, `exp()`, `floor()`, `ceil()`, `round()`, `pow()`,
`atan2()`.

Constants: `pi()`, `euler()`.

Random: `rand_int(lo, hi)`.

Time: `time_now()` (Unix seconds), `time_ms()` (milliseconds).

## Testing

```
test "addition works"
  assert 2 + 2 == 4

test "user creation"
  user := User(name: "Bob")
  assert user.name == "Bob"
```

Assertions: `assert(condition)`, `assert_eq(a, b)`, `assert_ne(a, b)`.

## Modules

```
use "other_file.ore"
```

Multi-file compilation is supported. The `use` statement imports definitions
from another `.ore` file.

## JSON

```
json_str := json_stringify(value)
parsed := json_parse(json_str)
```

Basic JSON serialization/deserialization to/from maps. No derive-based
serialization.

## Type Introspection

```
t := type_of(value)  -- returns a string like "Int", "Str", etc.
```

## Compilation Targets

- **LLVM backend**: JIT execution (`ore run`) and AOT compilation (`ore build`)
- **C backend**: Generates C code (`ore build --backend c`)
- **REPL**: Interactive mode (`ore repl`)

## What Is NOT Implemented

The following features appear in other docs but are **not yet available** in
the bootstrap compiler:

- **HTTP client/server** — no `http.get()`, no `http.server()`, no web framework
- **SQLite / database access** — no `sqlite.open()`, no `db.query()`
- **CLI argument parsing via `deriving(Cli)`** — no derive-based CLI generation
- **`deriving()` additional traits** — `deriving(Debug)`, `deriving(Eq)`, `deriving(Clone)`, `deriving(Serialize)` are implemented; `deriving(Cli)`, user-defined derives are not
- **Date/Time types** — no `DateTime`, `Date`, `Duration` (only `time_now()` for Unix timestamps)
- **Set type** — no `Set[T]` collection
- **Async/await** — no `async fn`, no `.await`, no structured async concurrency
- **Property-based testing** — no `for_all` / QuickCheck-style testing
- **Table-driven tests** — no `test "name" with [...]` syntax
- **Benchmarking** — no `bench` blocks
- **Package manager** — no `ore.toml`, no `ore deps add`
- **LSP** — no language server protocol support
- **Cross-compilation** — no `--target` flag
- **WebAssembly output** — no `--target wasm`
- **SQL parameterization magic** — string interpolation in SQL does not auto-parameterize
- **Structured concurrency** — no automatic task cancellation or scoped spawning
- **Cells (atomic shared state)** — no `cell` type
- **Parallel pipelines (`each|`)** — pipelines are sequential
- **`select` across channels** — no select statement
- **Implicit contexts (`with`)** — no capability/context system
- **`pub` visibility** — no enforced module-level access control
- **`@` annotations** — no `@jsonName`, `@short`, `@optional`, etc.
- **`ore doc`** — no documentation generation
- **`ore fmt`** — formatter exists but limited
- **Regex** — no regular expression support
- **TOML/YAML/MessagePack/CSV serialization** — only basic JSON
- **Int-keyed maps** — maps are string-keyed only
- **Type aliases** — no type alias syntax
- **Comparison chains** — no `0 < x < 100`
- **`crud` code generation** — no type-driven CRUD/database generation (but `deriving(Serialize)` provides the JSON foundation)
- **`fork` parallel pipelines** — `data | fork` with indented branches runs each branch in parallel and returns results as a list
