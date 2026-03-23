# Ore — Core Features

> **⚠ Many features below are ASPIRATIONAL.** They describe the language
> *design goals*, not what the bootstrap compiler implements today. Sections
> marked **[ASPIRATIONAL]** are not yet available. See
> [IMPLEMENTATION.md](IMPLEMENTATION.md) for what actually works.

## 1. First-Class Concurrency [PARTIALLY IMPLEMENTED]

> **What works today:** `spawn()`, `channel()`, `send()`/`recv()`, `sleep()`,
> `thread_join_all()` — using OS threads, not green threads.
>
> **Not yet implemented:** `async`/`.await`, structured concurrency, `select`,
> `each|` parallel pipelines, green threads, work-stealing scheduler.

Concurrency should be as easy to use as a for loop.

```
-- Async functions (colored, but lightweight)
fn fetchPage(url: Str) async -> Result[Str, HttpError] {
  resp := http.get(url).await?
  Ok(resp.body)
}

-- Spawn concurrent tasks
fn fetchAll(urls: List[Str]) async -> List[Str] {
  tasks := urls.map(url => spawn fetchPage(url))
  tasks.awaitAll().filterOk()
}

-- Channels for communication
fn pipeline() async {
  ch := Channel[Int](buffer: 100)

  spawn {
    for i in 0..1000 {
      ch.send(i)
    }
    ch.close()
  }

  for value in ch {
    process(value)
  }
}

-- Select across channels
select {
  msg from inbox => handleMessage(msg)
  tick from timer => updateState()
  _ after 5.seconds => timeout()
}
```

Structured concurrency: spawned tasks are tied to their parent scope. No
orphaned goroutines / tasks. Runtime is lightweight (green threads on a
work-stealing scheduler, like Go/Tokio but built-in).

## 2. Built-in HTTP [ASPIRATIONAL]

> **Not yet implemented.** There is no HTTP client or server in the bootstrap
> compiler. The examples below show the planned API design.

```
-- Client
resp := http.get("https://api.example.com/users").await?
users := resp.json[List[User]]()

http.post("https://api.example.com/users")
  .json(newUser)
  .header("Authorization", "Bearer {token}")
  .send().await?

-- Server
fn main() {
  app := http.server()

  app.get("/") { req =>
    Response.text("Hello, world!")
  }

  app.get("/users/{id}") { req =>
    id := req.param("id").parseInt()?
    user := db.findUser(id)?
    Response.json(user)
  }

  app.post("/users") { req =>
    user := req.json[CreateUser]()?
    created := db.createUser(user)?
    Response.json(created, status: 201)
  }

  app.listen(":8080")
}
```

No external framework needed for 90% of web services. Built into the standard
library. For complex apps, the primitives compose well.

## 3. Built-in JSON (and Serialization) [PARTIALLY IMPLEMENTED]

> **What works today:** `json_parse()` and `json_stringify()` for basic
> map-based JSON conversion.
>
> **Not yet implemented:** `deriving(Serialize)`, `toJson()`/`fromJson()`
> methods, `@jsonName`, `@optional` annotations, TOML/YAML/MessagePack/CSV.

```
-- Any type with `Serialize` can become JSON
type User deriving(Serialize) {
  name: Str
  email: Str
  age: Int
}

user := User(name: "Alice", email: "alice@example.com", age: 30)
jsonStr := user.toJson()         -- String
jsonBytes := user.toJsonBytes()  -- Bytes

-- Parsing
user := User.fromJson(jsonStr)?

-- Dynamic JSON (when you don't know the schema)
data := Json.parse(rawStr)?
name := data["users"][0]["name"].asStr()?

-- Field renaming, optional fields, custom parsing
type ApiResponse deriving(Serialize) {
  @jsonName("created_at")
  createdAt: DateTime

  @optional
  nickname: Str?
}
```

Serialization is derive-based. Covers JSON, TOML, YAML, MessagePack, and CSV
in the standard library.

## 4. Built-in CLI Argument Parsing [ASPIRATIONAL]

> **Not yet implemented.** There is no `deriving(Cli)` or automatic argument
> parsing. Use `args()` to get raw command-line arguments as a list of strings.

```
--- A tool that processes data files
type Args deriving(Cli) {
  --- Input file path
  input: Str

  --- Output file path (default: stdout)
  @short('o')
  output: Str = "-"

  --- Enable verbose logging
  @short('v')
  verbose: Bool = false

  --- Number of worker threads
  @short('j')
  jobs: Int = 4
}

fn main() {
  args := Args.parse()
  if args.verbose { log.setLevel(.Debug) }
  process(args.input, args.output, args.jobs)
}
```

Running `program --help` auto-generates:

```
A tool that processes data files

Usage: program <input> [options]

Arguments:
  <input>        Input file path

Options:
  -o, --output   Output file path (default: stdout) [default: -]
  -v, --verbose  Enable verbose logging
  -j, --jobs     Number of worker threads [default: 4]
  -h, --help     Show this help
```

Doc comments become help text. Types become validation. Defaults just work.

## 5. Built-in Testing [PARTIALLY IMPLEMENTED]

> **What works today:** `test "name" { ... }` blocks with `assert()`,
> `assert_eq()`, `assert_ne()`.
>
> **Not yet implemented:** table-driven tests (`test "name" with [...]`),
> property-based testing (`for_all`), async tests.

```
-- Tests live alongside code (or in separate files, your choice)
test "addition works" {
  assert 2 + 2 == 4
}

test "user creation" {
  user := User(name: "Bob", email: "bob@test.com", age: 25)
  assert user.name == "Bob"
  assert user.isValid()
}

-- Table-driven tests
test "fibonacci" with [
  (0, 0), (1, 1), (2, 1), (3, 2),
  (4, 3), (5, 5), (10, 55)
] { input, expected =>
  assert fib(input) == expected
}

-- Async tests
test "fetching works" async {
  resp := http.get(testServer.url("/health")).await?
  assert resp.status == 200
}

-- Property-based testing built in
test "sort is idempotent" for_all(list: List[Int]) {
  assert list.sort() == list.sort().sort()
}
```

Run with `ore test`. No test framework to install. No test runner to
configure.

## 6. First-Class Collections and Pipelines [PARTIALLY IMPLEMENTED]

> **What works today:** `List[T]` with full method suite (map, filter, reduce,
> sort, etc.), `Map[Str, V]` (string-keyed only), list comprehensions,
> pipeline operator `|`.
>
> **Not yet implemented:** `Set[T]`, `Map[K, V]` with non-string keys,
> destructuring assignments (`[first, ...rest] := list`,
> `{name, ...other} := record`).

```
numbers := [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

-- Pipeline operations chain naturally
result := numbers
  .filter(n => n % 2 == 0)
  .map(n => n * n)
  .sum()

-- Maps
counts := Map[Str, Int]()
for word in text.words() {
  counts[word] = counts.get(word, 0) + 1
}

-- Or more concisely
counts := text.words().countBy(w => w)

-- Sets
unique := Set.from(items)
common := setA & setB       -- intersection
all := setA | setB           -- union
diff := setA - setB          -- difference

-- Destructuring
[first, second, ...rest] := numbers
{name, age, ...other} := record

-- List comprehensions (when pipelines feel clunky)
squares := [x * x for x in 1..100 if x % 2 == 0]
```

## 7. Modules and Visibility [PARTIALLY IMPLEMENTED]

> **What works today:** `use "file.ore"` for multi-file compilation.
>
> **Not yet implemented:** `pub` visibility enforcement, dotted module paths
> (`use math.vector.{Vec2, dot}`), selective imports.

```
-- file: math/vector.lang

--- A 2D vector
pub type Vec2 {
  pub x: Float
  pub y: Float
}

pub fn dot(a: Vec2, b: Vec2) -> Float {
  a.x * b.x + a.y * b.y
}

-- Private helper (no pub = module-private)
fn normalize_internal(v: Vec2) -> Vec2 { ... }
```

```
-- file: main.lang

use math.vector.{Vec2, dot}

-- or
use math.vector -- then use as vector.Vec2, vector.dot
```

Simple. `pub` means public. No `pub` means module-private. No
`pub(crate)`, `protected`, `internal`, `package-private` complexity.

## 8. Database Access [ASPIRATIONAL]

> **Not yet implemented.** There is no SQLite or database access in the
> bootstrap compiler. The examples below show the planned API design.

```
-- Built-in SQLite, with compile-time query checking when schema is known
db := sqlite.open("app.db")

db.exec("""
  CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    email TEXT NOT NULL UNIQUE
  )
""")

-- Parameterized queries (SQL injection impossible by design)
users := db.query[User]("SELECT * FROM users WHERE age > {minAge}")
user := db.queryOne[User]("SELECT * FROM users WHERE id = {id}")?

-- Insert
db.exec("INSERT INTO users (name, email) VALUES ({name}, {email})")

-- Transactions
db.transaction { tx =>
  tx.exec("UPDATE accounts SET balance = balance - {amount} WHERE id = {from}")
  tx.exec("UPDATE accounts SET balance = balance + {amount} WHERE id = {to}")
}
```

String interpolation in SQL contexts automatically becomes parameterized
queries. The compiler knows the difference. `{id}` becomes `?` with `id`
as a bound parameter. You literally cannot write injectable SQL without
going out of your way to use raw string concatenation.

## 9. Date and Time [ASPIRATIONAL]

> **Not yet implemented.** There are no `DateTime`, `Date`, or `Duration`
> types. Only `time_now()` (Unix seconds) and `time_ms()` (milliseconds) are
> available.

```
now := DateTime.now()
birthday := Date(2000, 1, 15)
meeting := DateTime(2024, 3, 15, hour: 14, minute: 30, tz: "US/Eastern")

-- Arithmetic
tomorrow := now + 1.day
nextWeek := now + 7.days
duration := end - start  -- returns Duration

-- Formatting
now.format("YYYY-MM-DD HH:mm:ss")
now.format(.Iso8601)
now.format(.Rfc2822)

-- Parsing
dt := DateTime.parse("2024-03-15T14:30:00Z", .Iso8601)?
```

## 10. Process and I/O [PARTIALLY IMPLEMENTED]

> **What works today:** `print()`, `eprint()`, `readln()`, `file_read()`,
> `file_write()`, `file_append()`, `file_exists()`, `file_read_lines()`,
> `exec()`, `args()`, `env_get()`, `env_set()`, `exit()`.
>
> **Not yet implemented:** `run()` with structured result, pipe chaining,
> `readLines()` streaming iterator, `Path` type with methods.

```
-- Running external commands
result := run("git", "status")?
print(result.stdout)

-- Piping
output := run("cat", "data.txt")?.pipe("grep", "error")?.pipe("wc", "-l")?

-- File I/O
content := readFile("input.txt")?
writeFile("output.txt", processedContent)?

-- Streaming
for line in readLines("huge-file.txt") {
  if line.contains("ERROR") {
    log.warn(line)
  }
}

-- Path manipulation
path := Path("src") / "main" / "app.lang"
path.extension    -- "lang"
path.parent       -- Path("src/main")
path.exists()     -- Bool
```
