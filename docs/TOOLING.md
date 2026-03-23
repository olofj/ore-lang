# Ore — Tooling

> **⚠ Many features below are ASPIRATIONAL.** See which commands are
> available in [IMPLEMENTATION.md](IMPLEMENTATION.md).

Everything is built in. One binary. No ecosystem fragmentation.

## The `ore` Command

```
ore run file.ore         -- compile and run (cached, sub-second for most programs)
ore build                -- produce optimized binary (LLVM or C backend)
ore test                 -- run all tests
ore fmt                  -- format source code
ore check                -- type-check without building
ore repl                 -- interactive REPL
ore doc                  -- generate documentation [ASPIRATIONAL]
ore deps add foo         -- add a dependency [ASPIRATIONAL]
ore deps update          -- update dependencies [ASPIRATIONAL]
ore new project-name     -- scaffold a new project [ASPIRATIONAL]
ore bench                -- run benchmarks [ASPIRATIONAL]
ore lsp                  -- language server [ASPIRATIONAL]
```

## Project Structure

Small programs: just a `.ore` file. Run it directly.

```
-- hello.ore
fn main() {
  print("Hello, world!")
}
```

```
$ ore run hello.ore
Hello, world!
```

Larger projects:

```
myproject/
  ore.toml         -- project config and dependencies
  src/
    main.ore       -- entry point
    lib.ore         -- library root
    models/
      user.ore
    handlers/
      auth.ore
  tests/
    auth_test.ore
```

### ore.toml [ASPIRATIONAL]

```toml
[project]
name = "myproject"
version = "0.1.0"

[deps]
postgres = "2.1"
redis = "1.3"
```

That's it. No lock file drama — the lock file exists and is managed
automatically, but you never think about it.

## Formatter

One style. Period. No configuration. No debates.

The formatter is opinionated and fast. It runs on save. It runs in CI.
Everyone's code looks the same. This is a feature.

Similar to `gofmt` and `black` in philosophy, but built into the language
from day one.

## Compilation

The bootstrap compiler targets native code via **LLVM** (JIT and AOT) or
generates **C code** as an alternative backend.

Compilation modes:
- `ore run` — JIT execution via LLVM.
- `ore build` — AOT compilation to native binary via LLVM.
- `ore build --backend c` — Generate C code.

### Not yet implemented [ASPIRATIONAL]

- `ore build --target wasm` — WebAssembly output.
- Cross-compilation (`--target linux-amd64`, etc.).

## LSP [ASPIRATIONAL]

> **Not yet implemented.**

Planned to be built into the compiler. `ore lsp` would start it.
Planned features:
- Completions
- Hover information
- Go to definition
- Find references
- Rename refactoring
- Error diagnostics
- Inlay type hints

## REPL

```
$ ore repl
> 2 + 2
4
> fn fib n:Int -> Int
    if n < 2
      n
    else
      fib(n - 1) + fib(n - 2)
> fib(10)
55
```

The REPL supports core language features. HTTP and async are not available.

## Benchmarking [ASPIRATIONAL]

> **Not yet implemented.** `bench` blocks and `ore bench` are planned.

```
bench "sorting 10k elements"
  list := List.random(10000)
  list.sort()
```
