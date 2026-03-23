# Ore — Syntax Reference

## Design Principles

1. **Indentation-sensitive, no closing delimiters** — Blocks are defined by
   indentation. No `}`, no `end`. Every block-opening construct is syntactically
   unambiguous, so a formatter (`ore fmt`) can reconstruct indentation from
   keywords alone as a recovery mechanism.
2. **Minimal keywords** — Prefer symbols where they're universally understood
   (`+`, `=`, `->`, `|`), keywords where they add clarity (`fn`, `if`, `for`).
3. **Expression-oriented** — Everything is an expression. `if` returns a value.
   Blocks return their last expression.
4. **No semicolons** — Newlines terminate statements. Continuation is automatic
   when a line ends with an operator or open delimiter.
5. **Pipeline-first** — The `|` operator is a first-class spine. Data flows
   left to right, step by step. This is the primary abstraction, not the
   statement.

## The Basics

### Variables

```
-- Immutable binding (default)
x := 42
name := "hello"

-- Mutable binding
mut counter := 0
counter = counter + 1

-- Type annotation (rarely needed due to inference)
x: Int := 42

-- Compound assignment
counter += 1
counter -= 1
counter *= 2
counter /= 3
counter %= 7

-- Append to list
mut xs := [1, 2, 3]
xs ++= [4, 5]

-- Assign if not already set
mut x := 0
x ||= 42   -- x stays 0 (already set); if x were uninitialized, becomes 42
```

Immutable by default. `mut` is explicit and visible. `:=` declares and binds;
`=` reassigns a mutable variable.

### Functions

```
fn add a:Int b:Int -> Int
  a + b

-- Return type inferred when omitted
fn double x:Int
  x * 2

-- Multi-statement body
fn greet name:Str -> Str
  prefix := "Hello"
  "{prefix}, {name}!"

-- Default parameter values
fn greet name:Str greeting:Str = "Hello" -> Str
  "{greeting}, {name}!"

-- Single-expression shorthand
fn square x:Int = x * x

-- Generic functions
fn identity[T] x:T -> T
  x
```

Function definitions use `fn name param:Type ... -> ReturnType` followed by an
indented body. Parameters are positional. Default values use `=`.

Calls use parentheses: `add(3, 4)`, `greet("Alice")`, `greet("Bob", "Hi")`.

### Strings

```
-- Interpolation
name := "world"
greeting := "Hello, {name}!"

-- Triple-quoted multiline strings
multiline := """
  This is a multiline string.
  Common indentation is stripped.
  Values: {x}, {y}
"""

-- Adjacent string concatenation (auto-concat without operators)
msg := "Found " count " users in " elapsed "ms"

-- Equivalent using interpolation
msg := "Found {count} users in {elapsed}ms"
```

Interpolation uses `{expr}` directly inside string literals. Adjacent
expressions in string context auto-concatenate without `+`.

### Comments

```
-- This is a line comment
--- This is a doc comment (attaches to the next declaration)
```

## Control Flow

### If/Else

```
-- As expression
result := if x > 0
  "positive"
else
  "non-positive"

-- Inline form
result := if x > 0 then "positive" else "non-positive"

-- Multi-branch
if n < 0
  print "negative"
else if n == 0
  print "zero"
else
  print "positive"
```

No parentheses around conditions. Each branch is an indented block.

### Pattern Matching

Two equivalent forms for matching a value against patterns:

```
-- match keyword
match value
  0 -> "zero"
  1..10 -> "small"
  n if n > 100 -> "big: {n}"
  _ -> "other"

-- Colon operator (compact form)
value :
  0 -> "zero"
  1..10 -> "small"
  n if n > 100 -> "big: {n}"
  _ -> "other"
```

Both forms are exhaustive — the compiler reports an error for missing cases.

```
-- Destructuring enum variants
match shape
  Circle r -> 3.14159 * r * r
  Rect w h -> w * h

-- Or-patterns
match n
  1 | 2 | 3 -> "small"
  4 | 5 | 6 -> "medium"
  _ -> "other"

-- Guard patterns
match x
  0 -> "zero"
  _ if x > 0 -> "positive"
  _ -> "negative"

-- Match on Result/Option
safe_div(10, b) :
  Ok val -> print "Result: {val}"
  Err e -> print "Error: {e}"
```

Match arms use `->` (not `=>`). `=>` is reserved for lambdas.

### Loops

```
-- Iterate over a collection
for item in collection
  process(item)

-- With index
for i, item in collection
  print "{i}: {item}"

-- Range
for i in 0..10
  print i

-- Range with step
for i in 0..100 step 5
  print i

-- Iterate map
for k, v in myMap
  print "{k} = {v}"

-- While
while condition
  doWork()

-- Infinite loop (break to exit)
loop
  data := poll()
  if data.ready
    break data
```

`loop` is an expression — `break value` returns a value from it.

## Types

### Basic Types

```
Int       -- 64-bit signed integer (default)
Int32     -- 32-bit signed integer
Float     -- 64-bit float
Byte      -- unsigned 8-bit
Bool      -- true / false
Str       -- UTF-8 string, immutable, reference-counted
Char      -- Unicode scalar value
```

### Record Types

Record types (product types) use braces:

```
type Point { x: Float, y: Float }

-- With defaults
type Config { host: Str = "localhost", port: Int = 8080 }

-- Construction uses named fields
p := Point(x: 1.0, y: 2.0)
c := Config(port: 3000)  -- rest use defaults

-- Field access
print p.x
```

Anonymous record literals are inferred as named types when the structure
matches:

```
type Point { x: Int, y: Int }

-- Anonymous literal inferred as Point
p := {x: 3, y: 4}
print p.x
```

Tuples auto-convert to record types in typed context:

```
type Point { x: Int, y: Int }

p : Point := (3, 4)
print p.x   -- 3
print p.y   -- 4
```

### Enum Types (Sum Types)

Enum types use indented variants:

```
type Shape
  Circle(radius: Float)
  Rect(width: Float, height: Float)
  Triangle(a: Float, b: Float, c: Float)

-- Simple (no-payload) variants
type Color
  Red
  Green
  Blue

-- Built-in Option and Result
-- type Option[T] { Some(T), None }
-- type Result[T, E] { Ok(T), Err(E) }
```

### Deriving

Types can derive standard trait implementations:

```
type Point deriving(Eq, Debug, Clone, Serialize) { x: Int, y: Int }

-- Available derives: Debug, Eq, Clone, Serialize
-- Works for both record and enum types

type Color deriving(Eq, Debug)
  Red
  Green
  Blue
```

### Generics

```
fn identity[T] x:T -> T
  x

fn first[T] items:List[T] -> T?
  if items.len() == 0
    None
  else
    Some(items[0])
```

`T?` is sugar for `Option[T]`.

### Traits and Impl Blocks

```
trait Labeled
  fn label self:Self -> Str

impl Point
  fn distance_to self:Point other:Point -> Float
    dx := self.x - other.x
    dy := self.y - other.y
    sqrt(dx * dx + dy * dy)

  fn to_str self:Point -> Str
    "({self.x}, {self.y})"
```

Trait definitions use indented method signatures. `impl Type` adds methods to a
type. `impl Trait for Type` satisfies a trait.

### Null Safety

There is no null. Optional values use `Option[T]` (or `T?`).

```
fn find_user id:Int -> User?
  -- returns Some(user) or None

-- Safe chaining
name := find_user(42)?.name

-- Unwrap-or-panic (asserts non-None, crashes with source location if None)
name := find_user(42)!.name

-- Pattern match
find_user(42) :
  Some u -> greet(u)
  None -> print "not found"
```

The `!` postfix operator unwraps `Option` or `Result`, panicking with a
source-location message if the value is `None` or `Err`.

### Enum Methods

Built-in enum types (`Option`, `Result`) support method dispatch:

```
a := Some(5)
print a.is_some()       -- true
print a.unwrap_or(0)    -- 5
b := a.map(x => x * 2)  -- Some(10)
```

## Error Handling

```
-- Functions that can fail return Result
fn divide a:Int b:Int -> Result
  if b == 0
    Err("division by zero")
  else
    Ok(a / b)

-- The ? operator propagates errors
fn compute -> Result
  x := divide(10, 2)?
  y := divide(x, 3)?
  Ok(x + y)

-- Handle with pattern matching
divide(10, b) :
  Ok v -> print "Result: {v}"
  Err e -> print "Error: {e}"
```

Errors are values, not exceptions. The `?` operator unwraps `Ok` or returns
`Err` from the enclosing function.

## Pipelines

The `|` operator threads a value through a sequence of transformations:

```
-- Data flows left to right
result := [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
  | filter (n => n % 2 == 0)
  | map (n => n * n)

total := [10, 20, 30]
  | reduce (0, acc n => acc + n)
```

Each step receives the previous result as its first argument. Pipeline chains
can span multiple lines when continuation is indented under the starting value.

### Pipeline Dot Shorthand

Instead of explicit lambdas for field access, `.field` refers to the field of
whatever value is flowing through:

```
type Point { x: Int, y: Int }
type Line { start: Point, end_pt: Point }

p := Point(x: 3, y: 4)
a := p | .x           -- 3

line := Line(start: Point(x: 1, y: 2), end_pt: Point(x: 5, y: 6))
c := line | .start.x  -- 1
```

### Pipe-else (Inline Error Recovery)

```
result := find(id) | else defaultValue
```

`| else value` recovers from a `None` or `Err` by substituting `value`.

### Fork: Parallel Pipeline Branches

The `fork` construct splits a value into multiple independent pipeline
branches, executing each in parallel and collecting results:

```
fn double x:Int -> Int
  x * 2

fn add_ten x:Int -> Int
  x + 10

fn negate x:Int -> Int
  0 - x

results := 5 | fork
  double
  add_ten
  negate
-- results is a list: [10, 15, -5]
```

### Concurrent Map

```
-- Apply the pipeline to each element concurrently, collect results
results := urls | each| fetchPage
doubled := nums | each| (x => x * 2)
```

`each|` turns a pipeline step into a concurrent fan-out. The result is a list.

## Closures

```
double := (x => x * 2)
add := (x y => x + y)

-- Multi-line closure
transform := (n =>
  doubled := n * 2
  doubled + 1
)

-- Used inline
result := [1, 2, 3] | map (n => n * n)
[1, 2, 3].each(x =>
  y := x * 10
  print y
)
```

Closures capture free variables from the enclosing scope.

## Collections

### Lists

```
xs := [1, 2, 3, 4, 5]

-- Index access
print xs[0]
print xs.len()

-- Mutation (requires mutable list)
mut ys := [1, 2, 3]
ys[1] = 99

-- Append
mut zs := [1, 2, 3]
zs ++= [4, 5]

-- Common methods
xs.map(x => x * 2)
xs.filter(x => x > 2)
xs.reduce(0, acc x => acc + x)
xs.each(x => print x)
xs.join(", ")
xs.sum()
xs.count()
xs.min()
xs.max()
xs.any(x => x > 3)
xs.all(x => x > 0)
xs.find(x => x > 3)
xs.sort()
```

List comprehensions:

```
squares := [x * x for x in 0..5]
evens := [x for x in 0..10 if x % 2 == 0]
```

### Maps

```
m := {"a": 1, "b": 2, "c": 3}

-- Access
print m.get("a")
print m.get_or("z", 0)
print m.len()
print m.contains("a")

-- Mutation
m.set("d", 4)
m.remove("a")

-- Iteration
for k, v in m
  print "{k}: {v}"

-- Transform
m.entries()
m.values()
m.keys()
m.merge(other)
```

Maps are string-keyed only in the bootstrap compiler.

## Concurrency

```
-- Spawn a task
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

Concurrency uses OS threads, not green threads. `spawn` runs a function
in a new thread. Channels are typed and multi-producer/multi-consumer.

## Modules

```
-- Import another file
use "other_file.ore"

-- Imported names are available directly
```

Multi-file compilation is supported. The `use` statement imports definitions
from another `.ore` file.

## Testing

```
test "addition works"
  assert 2 + 2 == 4

test "user creation"
  user := User(name: "Bob")
  assert user.name == "Bob"
```

Assertions: `assert(condition)`, `assert_eq(a, b)`, `assert_ne(a, b)`.

## Quick Reference

| Feature | Syntax | Example |
|---------|--------|---------|
| Function def | `fn name param:Type -> Ret` | `fn add a:Int b:Int -> Int` |
| Function call | parentheses | `add(1, 2)` |
| Match arm | `->` | `0 -> "zero"` |
| Lambda | `=>` | `n => n * 2` |
| Inline if | `if ... then ... else ...` | `if x > 0 then x else -x` |
| Block if | indented body | `if x > 0` + newline + indent |
| Inline match | `subject :` | `x : None : Some(x)` |
| Block match | `match subject` | `match x` + newline + arms |
| Record type | inline braces | `type Point { x:Float, y:Float }` |
| Enum type | indented variants | `type Color` + newline + variants |
| Pipe | `\|` | `5 \| double \| double` |
| Pipe dot | `\| .field` | `p \| .x` |
| Fork | `\| fork` | `5 \| fork` + newline + branches |
| Unwrap | `!` | `opt!` panics if None |
| Derive | `deriving(...)` | `type T deriving(Eq) { ... }` |
| Append | `++=` | `xs ++= [4, 5]` |

---

## Planned / Not Yet Implemented

The following features are part of the Ore vision but are not yet implemented
in the compiler.

### Implicit Context Injection (`with`)

Functions can declare capability requirements. The caller provides these
implicitly via a `with` block:

```
fn get_user id:Int -> User with Db, Log
  log.info "fetching user {id}"
  db.query_one "SELECT * FROM users WHERE id = {id}"

fn main
  ctx := { db: sqlite.open("app.db"), log: Logger.stdout() }
  with ctx
    server.get "/users/{id}" req ->
      get_user(req.param("id") | parse_int)
```

### Type-Driven Code Generation (`crud`)

First-class macros that inspect a type at compile time and generate boilerplate:

```
type Todo { id: Int, title: Str, done: Bool = false }

crud Todo "todos"
  | server.mount "/todos"
```

### Comparison Chains

```
0 < x < 100         -- true if x is in range (0, 100)
a == b == c          -- true if all three are equal
```

### Async/Await

Structured async concurrency is planned but not yet implemented.

### HTTP Client/Server, SQLite, Package Manager

See [IMPLEMENTATION.md](IMPLEMENTATION.md) for the full list of what is and
is not yet available.
