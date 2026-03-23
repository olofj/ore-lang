# Ore — Syntax Reference (Canonical)

> **This is the canonical syntax document for the Ore language.** It describes
> the syntax that the compiler implements. For what works today in the bootstrap
> compiler, see [IMPLEMENTATION.md](IMPLEMENTATION.md). Some features shown
> here (like `deriving`, async/await, HTTP, SQLite) are aspirational and not
> yet implemented in the bootstrap compiler.
>
> For exploratory syntax ideas (parallel pipelines, implicit contexts, CRUD
> generation), see [RADICAL.md](RADICAL.md).

## Design Principles

1. **Indentation-sensitive** — Blocks are delimited by indentation (like Python).
   No braces required for function bodies, control flow, or type declarations.
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

-- Compound assignment
counter += 1
counter -= 2
counter *= 3
counter /= 4
counter %= 5

-- Type annotation (rarely needed due to inference)
x: Int := 42
```

Immutable by default. `mut` is explicit and visible. `:=` for binding
(declaration + assignment), `=` for reassignment of mutable bindings.

### Functions

```
fn add a:Int b:Int -> Int
  a + b

-- Return type inferred for private functions
fn double x:Int
  x * 2

-- Multi-expression body
fn greet name:Str -> Str
  prefix := "Hello"
  "{prefix}, {name}!"

-- No parameters
fn main
  print "hello"

-- Default parameter values
fn greet name:Str greeting:Str = "Hello" -> Str
  "{greeting}, {name}!"

-- Generic functions
fn identity[T] x:T -> T
  x
```

Function signatures use space-separated `name:Type` pairs — no parentheses
around parameter lists. Return type follows `->`. Body is indented on the
next line.

Public function signatures require return types (documentation + API contract).
Private functions infer everything.

**Calling** functions uses parentheses: `add(1, 2)`, `greet("Alice")`.

### String Interpolation

```
name := "world"
greeting := "Hello, {name}!"
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

Two forms: inline (single expression) and block (multi-line).

**Inline form** (uses `then` keyword):
```
result := if x > 0 then "positive" else "non-positive"
sign := if n > 0 then "positive" else if n < 0 then "negative" else "zero"
abs_n := if n < 0 then -n else n
```

**Block form** (uses indentation):
```
if condition
  doSomething()
else if other
  doOther()
else
  fallback()
```

Block if/else is also an expression — the last expression in each branch
is the result:
```
result := if x > 0
  "positive"
else
  "non-positive"
```

No parentheses around conditions.

### Pattern Matching

Two forms: `match` keyword and inline colon operator.

**`match` keyword** (multi-line, indented arms):
```
match value
  0 -> "zero"
  1 -> "one"
  _ -> "other"

-- With guards
match x
  0 -> "zero"
  _ if x > 0 -> "positive"
  _ -> "negative"

-- Or-patterns
match x
  1 | 2 | 3 -> "small"
  4 | 5 | 6 -> "medium"
  _ -> "other"

-- Destructuring enum variants
match shape
  Circle r -> 3.14159 * r * r
  Rect w h -> w * h
```

**Colon operator** (inline match):
```
result := value :
  pattern -> expression
  other -> expression

-- Ternary-like (two-arm match on bool)
b == 0 : Err(-1) : Ok(a / b)
```

Match arms use `->` (not `=>`). `=>` is reserved for lambdas.
Matches are exhaustive by default — compiler error if you miss a case.

### Loops

```
-- For over range
for i in 0..10
  print i

-- For with step
for i in 0..10 step 2
  print i

-- For over collection
for item in collection
  process(item)

-- For with key-value
for k, v in map
  print k

-- While
while condition
  doWork()

-- Infinite loop (break to exit)
loop
  x = x + 1
  if x >= 3
    break
```

`break` and `continue` work as expected.

## Lambdas and Pipes

Lambdas use `=>` (fat arrow):
```
nums.map(n => n * 2)
nums.filter(n => n > 3)
nums.each(x => print x)
```

Pipe operator `|` threads values through functions:
```
result := 5 | double | double

-- With inline lambdas
y := 10 | (x => x + 5) | (x => x * 2)

-- Multi-line pipes
result := [1, 2, 3, 4, 5]
  | filter (n => n % 2 == 0)
  | map (n => n * n)
```

### List Comprehensions

```
squares := [x * x for x in 0..5]
evens := [x for x in 0..10 if x % 2 == 0]
doubled := [n * 2 for n in nums]
```

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
-- Records (product types) — inline brace syntax
type Point { x:Float, y:Float }
type Config { host:Str, port:Int }

-- Construction with named fields
p := Point(x: 1.0, y: 2.0)
c := Config(host: "localhost", port: 3000)

-- Enums (sum types) — indented variants
type Shape
  Circle(radius: Float)
  Rect(width: Float, height: Float)

-- Simple enums
type Color
  Red()
  Green()
  Blue()

-- Option and Result are built-in
type Option
  Some(value)
  None

type Result
  Ok(value)
  Err(error)
```

### Generics

```
fn identity[T] x:T -> T
  x

fn double[T] x:T -> T
  x + x
```

`T?` is sugar for `Option[T]`. Clean and universally understood.

### Traits (Interfaces)

```
trait Describable
  fn describe self:Self -> Int

-- Implementation for a specific type
impl Describable for Dog
  fn describe self:Dog -> Int
    self.age
```

### Methods (impl blocks)

```
type Point { x:Float, y:Float }

impl Point
  fn magnitude self:Point -> Float
    self.x * self.x + self.y * self.y

  fn translate self:Point dx:Float dy:Float -> Point
    Point(x: self.x + dx, y: self.y + dy)

-- Usage
p := Point(x: 3.0, y: 4.0)
print p.magnitude()
p2 := p.translate(1.0, 1.0)
```

### Null Safety

There is no null. Period. Optional values use `Option` (or `T?`).

```
fn safe_div a:Int b:Int -> Option
  b == 0 : None : Some(a / b)

-- Must handle the None case
opt :
  Some val -> val
  None -> default
```

## Error Handling

```
-- Functions that can fail return Result
fn safe_div a:Int b:Int -> Result
  b == 0 : Err(-1) : Ok(a / b)

-- Match on Result
r :
  Ok val -> val
  Err e -> e
```

Errors are values, not exceptions.

## Concurrency

```
-- Spawn a concurrent task
spawn greet()
```

## Modules

```
-- Import another Ore file
use "math.ore"

fn main
  print add(3, 4)
```

## Quick Reference: Syntax Distinctions

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
