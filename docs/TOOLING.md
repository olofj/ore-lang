# Ore — Tooling

Everything is built in. One binary. No ecosystem fragmentation.

## The `ore` Command

```
ore run file.ore         -- compile and run (cached, sub-second for most programs)
ore build                -- produce optimized binary
ore test                 -- run all tests
ore fmt                  -- format all code (one style, no config, not negotiable)
ore check                -- type-check without building
ore doc                  -- generate documentation
ore deps add foo         -- add a dependency
ore deps update          -- update dependencies
ore new project-name     -- scaffold a new project
ore repl                 -- interactive REPL
ore bench                -- run benchmarks
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

### ore.toml

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

The compiler targets native code via a lightweight backend (think Go's
compiler speed, not LLVM's). Debug builds compile in milliseconds. Release
builds take a few seconds and produce optimized binaries.

Compilation modes:
- `ore run` — JIT-like experience. Caches compilation, re-runs instantly if
  source hasn't changed.
- `ore build` — Optimized native binary. Static by default.
- `ore build --target wasm` — WebAssembly output.

Cross-compilation works out of the box:
```
ore build --target linux-amd64
ore build --target darwin-arm64
ore build --target windows-amd64
```

## LSP

Built into the compiler. `ore lsp` starts it. Works with every editor.
Provides:
- Completions
- Hover information
- Go to definition
- Find references
- Rename refactoring
- Error diagnostics
- Inlay type hints

## REPL

```
$ lang repl
> 2 + 2
4
> fn fib(n: Int) { if n < 2 { n } else { fib(n-1) + fib(n-2) } }
> fib(10)
55
> use http
> http.get("https://httpbin.org/ip").await?.json()
{"origin": "1.2.3.4"}
```

Full language support. Async works. Imports work. Useful for exploration.

## Benchmarking

```
bench "sorting 10k elements" {
  list := List.random(10000)
  list.sort()
}

bench "map lookup" {
  map := Map.from(entries)
  map.get("key")
}
```

```
$ lang bench
sorting 10k elements ... 1.23 ms/iter (± 0.05 ms)
map lookup            ... 45 ns/iter (± 2 ns)
```
