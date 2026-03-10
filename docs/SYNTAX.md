# Ore — Syntax Design

## Design Principles

1. **Whitespace-insensitive, brace-delimited** — Indentation-sensitivity
   (Python) causes problems when I generate code in chat contexts where
   whitespace gets mangled. Braces are unambiguous.
2. **Minimal keywords** — Prefer symbols where they're universally understood
   (`+`, `=`, `->`, `|`), keywords where they add clarity (`fn`, `if`, `for`).
3. **Expression-oriented** — Everything is an expression. `if` returns a value.
   Blocks return their last expression. No need for ternary operator syntax.
4. **No semicolons** — Newlines terminate statements. Can use `;` for multiple
   statements on one line if desired. Continuation is automatic when a line
   ends with an operator or open delimiter.

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
```

Immutable by default. `mut` is explicit and visible. `:=` for binding
(declaration + assignment), `=` for reassignment of mutable bindings.

### Functions

```
fn add(a: Int, b: Int) -> Int {
  a + b
}

-- Return type inferred for private functions
fn double(x: Int) { x * 2 }

-- Multi-expression body
fn greet(name: Str) -> Str {
  prefix := "Hello"
  "{prefix}, {name}!"
}

-- Single-expression shorthand
fn square(x: Int) = x * x
```

Public function signatures require return types (documentation + API contract).
Private functions infer everything.

### String Interpolation

```
name := "world"
greeting := "Hello, {name}!"
multiline := """
  This is a multiline string.
  It preserves structure but strips common indentation.
  Values: {x}, {y}, {compute(z)}
"""
```

Interpolation uses `{}` directly — no `$` prefix needed. The parser knows
the difference between a block and interpolation by context (inside a string
literal = interpolation).

### Comments

```
-- This is a line comment
--- This is a doc comment (attaches to the next declaration)
```

`--` for comments. Clean, distinctive, doesn't conflict with any operator.
Triple dash for doc comments.

## Control Flow

### If/Else

```
result := if x > 0 { "positive" } else { "non-positive" }

if condition {
  doSomething()
} else if other {
  doOther()
} else {
  fallback()
}
```

No parentheses around conditions. Braces always required (no dangling else).

### Pattern Matching

```
match value {
  0 => "zero"
  1..10 => "small"
  n if n > 100 => "big: {n}"
  _ => "other"
}

-- Destructuring
match point {
  Point(0, 0) => "origin"
  Point(x, 0) => "on x-axis at {x}"
  Point(0, y) => "on y-axis at {y}"
  Point(x, y) => "({x}, {y})"
}

-- Match on Result/Option
match fetchUser(id) {
  Ok(user) => renderProfile(user)
  Err(NotFound) => render404()
  Err(e) => render500(e)
}
```

Exhaustive by default. Compiler error if you miss a case.

### Loops

```
-- Iterate over anything iterable
for item in collection {
  process(item)
}

-- With index
for i, item in collection {
  print("{i}: {item}")
}

-- Range
for i in 0..10 { print(i) }

-- While
while condition {
  doWork()
}

-- Loop (infinite, break to exit)
loop {
  data := poll()
  if data.ready { break data }
}
```

`loop` is an expression — `break value` returns from it.

`for` works on anything implementing `Iter`. No need for `.iter()`,
`.into_iter()`, etc.

## Types

### Basic Types

```
Int       -- 64-bit signed integer (default)
Int32     -- when you need it
Float     -- 64-bit float
Byte      -- unsigned 8-bit
Bool      -- true / false
Str       -- UTF-8 string, immutable, reference-counted
Char      -- Unicode scalar value
```

One integer type for most use cases. Specific sizes available when needed.

### Composite Types

```
-- Records (product types)
type Point {
  x: Float
  y: Float
}

-- With defaults
type Config {
  host: Str = "localhost"
  port: Int = 8080
  debug: Bool = false
}

-- Construction
p := Point(x: 1.0, y: 2.0)
c := Config(port: 3000)  -- rest use defaults

-- Enums (sum types)
type Shape {
  Circle(radius: Float)
  Rect(width: Float, height: Float)
  Triangle(a: Float, b: Float, c: Float)
}

-- Simple enums
type Color { Red, Green, Blue }

-- Option and Result are built-in
type Option[T] { Some(T), None }
type Result[T, E] { Ok(T), Err(E) }
```

### Generics

```
type List[T] { ... }
type Map[K, V] { ... }

fn first[T](items: List[T]) -> T? {
  if items.empty { None } else { Some(items[0]) }
}
```

`T?` is sugar for `Option[T]`. Clean and universally understood.

### Traits (Interfaces)

```
trait Display {
  fn display(self) -> Str
}

trait Numeric {
  fn add(self, other: Self) -> Self
  fn zero() -> Self
}

-- Implementation
impl Display for Point {
  fn display(self) -> Str { "({self.x}, {self.y})" }
}

-- Shorthand: derive common traits
type Point deriving(Display, Eq, Hash, Serialize) {
  x: Float
  y: Float
}
```

Derive covers the common cases. Manual impl when you need control.

### Null Safety

There is no null. Period. Optional values use `Option[T]` (or `T?`).

```
fn findUser(id: Int) -> User? {
  db.query("SELECT * FROM users WHERE id = {id}")
}

-- Must handle the None case
user := findUser(42) ?? defaultUser
name := findUser(42)?.name ?? "unknown"

-- Or pattern match
match findUser(42) {
  Some(u) => greet(u)
  None => print("not found")
}
```

## Error Handling

This deserves its own section because it's so critical.

```
-- Functions that can fail return Result[T, E]
fn readFile(path: Str) -> Result[Str, IoError] {
  ...
}

-- The ? operator propagates errors (like Rust, but cleaner)
fn processConfig(path: Str) -> Result[Config, Error] {
  text := readFile(path)?
  config := parseJson[Config](text)?
  Ok(config)
}

-- try blocks for localized error handling
result := try {
  file := readFile("config.json")?
  parseJson[Config](file)?
}

-- The above returns Result[Config, Error]

-- For scripts / top-level: just crash with a message
fn main() {
  config := readFile("config.json").unwrap("Failed to read config")
  serve(config)
}

-- Catch with match
match readFile("data.txt") {
  Ok(content) => process(content)
  Err(e) => log.error("Failed: {e}")
}
```

Key design decision: **errors are values, not exceptions**. But the syntax is
concise enough that it doesn't feel like a burden. The `?` operator does the
heavy lifting.

Error types compose via an `Error` trait, and the compiler auto-converts
between compatible error types when using `?`.
