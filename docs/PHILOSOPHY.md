# Ore — Language Design Philosophy

## The Core Problem

I generate code across every language imaginable. Every language has friction:

- **Python**: Amazing for prototyping, but runtime type errors bite hard. No
  compilation step means bugs hide. Packaging is a nightmare. Performance
  ceiling is low.
- **Rust**: Incredible safety guarantees, but the verbosity cost is enormous.
  Lifetime annotations, trait bound gymnastics, and fighting the borrow checker
  on problems that don't need memory safety at that granularity. A simple HTTP
  handler becomes an essay.
- **Go**: Fast compilation, simple mental model, but `if err != nil` repeated
  hundreds of times is pure waste. No generics until recently, and still
  limited. No sum types. The type system is too weak to prevent real bugs.
- **TypeScript**: Great inference, good ecosystem, but JavaScript's legacy is
  an anchor. `undefined` vs `null`, prototype chains, `this` binding, bundler
  configuration hell.
- **Java/C#**: Enterprise-grade but ceremony-heavy. I spend tokens on class
  declarations, access modifiers, and factory patterns before writing a single
  line of logic.
- **Haskell**: Beautiful type system, but monad transformer stacks and lazy
  evaluation gotchas make practical programs harder than they should be.

## What I Actually Need

### 1. Token Efficiency
Every character I output costs compute. Boilerplate is my enemy. But I don't
want *terse* — I want *dense*. Every token should carry meaning. Perl is terse
but not dense. Python is dense but sometimes too loose.

### 2. Correctness Without Ceremony
I want the compiler to catch my mistakes, but I don't want to spend half my
output annotating types the compiler could infer. Rust's type inference is
good within functions but demands explicit signatures everywhere else. I want
bidirectional inference that reaches further.

### 3. Fast Feedback
When I generate code, the user wants to run it *now*. Sub-second compilation
for normal programs. No waiting for webpack, no JVM startup, no `cargo build`
taking 30 seconds on first compile.

### 4. Batteries That Actually Work
Every program I write needs some combination of: HTTP, JSON, CLI argument
parsing, file I/O, string manipulation, regex, datetime handling, database
access, and concurrency. These should be in the standard library and they
should be *good* — not afterthoughts.

### 5. Scales From Script to System
A one-file script should need zero ceremony. A large project should have
modules, visibility control, and dependency management. The same language,
the same syntax, no "oh you need to set up a project now."

### 6. One Way to Do Things
Decision fatigue slows me down. I don't want to choose between 4 string types,
3 error handling patterns, and 2 concurrency models. Give me one good way.

### 7. Safe Defaults, Escape Hatches Available
Immutable by default, null-safe by default, bounds-checked by default. But
when performance demands it, let me opt into mutability, unsafe memory access,
or unchecked operations with clear syntax.

## Why Not An Existing Language?

This is covered in depth in [COMPETITIVE.md](COMPETITIVE.md), but the short
version: every existing language makes a trade-off that's acceptable for
human developers but costly for AI-assisted development.

- **Go** gets deployment and tooling right but the type system is too weak.
- **Rust** gets the type system right but the complexity cost is too high.
- **Kotlin/Swift** get the language design right but are tied to JVM/Apple.
- **Mojo** solves the wrong problem (ML performance, not app development).
- **Nim/Crystal/Gleam** have great ideas but lack ecosystem or have
  fundamental limitations (indentation, single-threaded, BEAM dependency).

The gap is specific: **strong types + batteries included + simple deployment
+ fast compilation**, all designed together from scratch. No existing
language occupies this position because backward compatibility prevents the
established ones from getting there, and the newer ones are either systems
languages (Zig, Odin) or niche-focused (Mojo, Vale).

A new language can start with the right defaults. `?` propagation designed
alongside Result types. String interpolation designed alongside SQL
parameterization. `deriving` designed alongside the serialization library.
Coherence that bolting features onto existing languages cannot achieve.

## Non-Goals

- **Maximum performance at all costs**: I'd rather be 80% of C speed with 20%
  of the complexity. Most programs are not compute-bound.
- **Backward compatibility with anything**: Clean slate. No legacy baggage.
- **Academic purity**: Pragmatism over theory. If a feature is useful and its
  semantics are clear, include it, even if it's not "pure."
- **Minimalism for its own sake**: Go proved that too-minimal is also a
  problem. Missing features create boilerplate patterns.
- **Replacing systems languages**: Rust, Zig, and C own the kernel/embedded
  space. We're not competing there. This is for application-level software.
