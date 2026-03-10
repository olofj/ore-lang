# Ore — Design Trade-offs and Rationale

## Decisions I Wrestled With

### 1. Garbage Collection vs Ownership

**Chosen: Tracing GC with value types and escape analysis**

Rust's ownership model prevents an entire class of bugs, but it also
prevents me from writing code quickly. The borrow checker is a tax on
every data structure decision. For the vast majority of programs I write
(web services, CLI tools, data processors), GC pauses measured in
microseconds are completely irrelevant.

The compiler aggressively stack-allocates through escape analysis. Value
types (records without references) are passed by value. Strings are
reference-counted (predictable, no GC involvement). The GC handles the
rest with a concurrent, low-pause collector.

For the rare performance-critical section, `@noalloc` blocks give you
stack-only guarantees with compiler verification.

**What I gave up**: Deterministic destruction, zero-cost abstractions for
systems programming, suitability for kernel/embedded work.

**What I gained**: Simple mental model, fast iteration, no lifetime
annotation tax.

### 2. Async Coloring

**Chosen: Colored async (explicit `async`/`.await`)**

I considered Go's approach (everything is implicitly concurrent), but
explicit async has real advantages:
- The type system tracks what can suspend
- You can't accidentally call an async function in a sync context
- Performance is more predictable
- Debugging is easier — you know where suspension points are

The `.await` syntax makes suspension points visible in the code. This is
worth the small annotation cost.

**Mitigation**: The runtime is lightweight. `async fn main()` works at the
top level. Most programs only need to think about async at the HTTP handler
level, where it's natural.

### 3. No Exceptions

**Chosen: Result types with `?` propagation**

Exceptions are invisible control flow. They make it impossible to know
which functions can fail by looking at their signatures. They make
resource cleanup fragile. They make error handling feel like an
afterthought.

Result types with `?` are almost as concise as exceptions for the "just
propagate it" case, but explicit about where errors can occur and what
errors are possible.

**What I gave up**: The convenience of "just throw and catch at the top."

**What I gained**: Every function's signature tells you if it can fail and
how. Error handling is always visible. Composition works cleanly.

### 4. One String Type

**Chosen: Single `Str` type — immutable, UTF-8, reference-counted**

Rust has `String`, `&str`, `&[u8]`, `OsStr`, `CStr`, `Cow<str>`. C++ has
`std::string`, `std::string_view`, `const char*`, `std::wstring`.

I chose one string type. It's always valid UTF-8. It's immutable (mutations
create new strings, but small-string optimization and CoW make this fast
in practice). Reference counting means no GC involvement and predictable
cleanup.

For byte-level manipulation, use `Bytes`. For OS interaction, conversion
happens at the boundary.

**What I gave up**: Maximum performance for string-heavy inner loops.

**What I gained**: Never thinking about which string type to use. Ever.

### 5. Significant Formatting (gofmt-style)

**Chosen: Mandatory formatting, no configuration**

This is the single biggest productivity boost for me. When I generate code,
I don't want to guess whether the project uses tabs or spaces, 2-indent
or 4-indent, trailing commas or not. One format. Always.

**What I gave up**: Developer preference for code style.

**What I gained**: Every file in every project looks the same. Code review
focuses on logic, not style. I never generate wrongly-formatted code.

### 6. No Inheritance

**Chosen: Composition + traits, no class hierarchy**

Inheritance creates deep coupling and fragile base class problems. I've
generated thousands of programs and inheritance causes more bugs than it
prevents. Composition is more explicit, more flexible, and easier to
reason about.

```
-- Instead of inheriting from Animal:
type Dog {
  name: Str
  breed: Str
}

impl Animal for Dog {
  fn speak(self) -> Str { "Woof!" }
  fn name(self) -> Str { self.name }
}

-- Embedding for code reuse
type Server {
  config: Config
  ...Logger  -- embed Logger's fields and methods
}
```

### 7. SQL String Interpolation Magic

**Chosen: Interpolation in SQL contexts auto-parameterizes**

This is unusual and potentially confusing. But SQL injection is the most
common security vulnerability in web applications, and parameterized
queries are the solution that nobody uses because the syntax is annoying.

```
-- This looks like interpolation but compiles to a parameterized query
db.query("SELECT * FROM users WHERE name = {name}")
-- Becomes: SELECT * FROM users WHERE name = ?  [params: name]
```

The compiler knows that `db.query()` expects SQL. The interpolation syntax
is the same, but the semantics are safe-by-default.

**What I gave up**: Consistency of interpolation semantics across contexts.

**What I gained**: SQL injection is impossible by default.

## The Language I Didn't Build

This language is deliberately *not*:

- **A systems language**: Use Rust or C for kernels, drivers, and embedded.
- **A scripting language**: It compiles to native code. It has types.
  But it *feels* as easy as scripting.
- **A functional language**: It borrows good ideas (immutability, pattern
  matching, expression orientation) but doesn't enforce purity.
- **An OOP language**: No classes, no inheritance. But it has methods and
  polymorphism through traits.

It sits in the space where Go, Kotlin, and Swift live — pragmatic, typed,
compiled, productive — but optimized for the specific workflow of an AI
that generates complete programs from specifications.

## Comparison Matrix

| Feature              | This Lang | Go    | Rust  | Kotlin | Swift  | Mojo   | Nim    | Gleam  |
|----------------------|-----------|-------|-------|--------|--------|--------|--------|--------|
| Type inference       | Strong    | Weak  | Good  | Strong | Strong | Medium | Strong | Good   |
| Null safety          | Yes       | No    | Yes   | Yes    | Yes    | Partial| Partial| Yes    |
| Error handling       | Result+?  | Multi | Result| Except | Result | Except | Except | Result |
| Sum types            | Yes       | No    | Yes   | Yes    | Yes    | No     | Yes    | Yes    |
| Generics             | Yes       | Basic | Yes   | Yes    | Yes    | Partial| Yes    | Yes    |
| Built-in HTTP        | Yes       | Yes   | No    | No     | No     | No     | Yes    | No     |
| Built-in JSON        | Yes       | Yes   | No    | No*    | No     | No     | Yes    | Yes    |
| Built-in testing     | Yes       | Yes   | Yes   | No     | Yes    | No     | Yes    | No     |
| Compilation speed    | Fast      | Fast  | Slow  | Medium | Medium | Medium | Fast   | Fast   |
| GC pauses            | Low       | Yes   | None  | Yes    | None†  | None   | Config | Yes    |
| Single binary        | Yes       | Yes   | Yes   | No‡    | Yes    | Yes    | Yes    | No     |
| Whitespace-safe      | Yes       | Yes   | Yes   | Yes    | Yes    | No     | No     | Yes    |
| Learning curve       | Low       | Low   | High  | Medium | Medium | Medium | Medium | Medium |

*Kotlin has kotlinx.serialization but it's external
†Swift uses ARC, not GC — no pauses but retain cycle risk
‡GraalVM native-image exists but is slow and limited

For a detailed analysis of each language, see [COMPETITIVE.md](COMPETITIVE.md).
