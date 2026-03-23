# Ore — Complete Examples

> **⚠ Most examples below are ASPIRATIONAL.** They show what the language
> *aims to look like*, not what the bootstrap compiler supports today.
> Examples use brace syntax and features (HTTP, SQLite, `deriving`, `async`)
> that are not yet implemented. See [IMPLEMENTATION.md](IMPLEMENTATION.md)
> for what actually works.

Showing what real programs look like in this language.

## Example 1: CLI Tool — Word Frequency Counter [ASPIRATIONAL]

> Uses `deriving(Cli)`, brace syntax, and methods not available in the
> bootstrap compiler.

```
--- Count word frequencies in text files
type Args deriving(Cli) {
  --- Input files to analyze
  files: List[Str]

  --- Show top N results
  @short('n')
  top: Int = 10

  --- Minimum word length
  @short('m')
  minLen: Int = 1
}

fn main() {
  args := Args.parse()

  counts := args.files
    .flatMap(f => readFile(f).unwrap("Cannot read {f}").words())
    .filter(w => w.len >= args.minLen)
    .map(w => w.toLower())
    .countBy(w => w)
    .entries()
    .sortBy(e => -e.value)
    .take(args.top)

  for entry in counts {
    print("{entry.value:>8} {entry.key}")
  }
}
```

That's the whole program. ~20 lines for a useful CLI tool with argument
parsing, file I/O, text processing, sorting, and formatted output.

## Example 2: REST API [ASPIRATIONAL]

> Uses HTTP server, SQLite, `deriving(Serialize)` — none implemented.

```
use db.sqlite
use log

type Todo deriving(Serialize, Eq) {
  id: Int
  title: Str
  done: Bool = false
}

type CreateTodo deriving(Serialize) {
  title: Str
}

fn main() {
  db := sqlite.open("todos.db")
  db.exec("CREATE TABLE IF NOT EXISTS todos (
    id INTEGER PRIMARY KEY,
    title TEXT NOT NULL,
    done BOOLEAN DEFAULT FALSE
  )")

  app := http.server()
  app.state(db)

  app.get("/todos") { req =>
    todos := req.state[Db].query[Todo]("SELECT * FROM todos")
    Response.json(todos)
  }

  app.post("/todos") { req =>
    input := req.json[CreateTodo]()?
    req.state[Db].exec("INSERT INTO todos (title) VALUES ({input.title})")
    Response.json(input, status: 201)
  }

  app.put("/todos/{id}/done") { req =>
    id := req.param("id").parseInt()?
    req.state[Db].exec("UPDATE todos SET done = TRUE WHERE id = {id}")
    Response.ok()
  }

  app.delete("/todos/{id}") { req =>
    id := req.param("id").parseInt()?
    req.state[Db].exec("DELETE FROM todos WHERE id = {id}")
    Response.ok()
  }

  log.info("Listening on :8080")
  app.listen(":8080")
}
```

A complete CRUD API in ~40 lines. No framework, no ORM, no boilerplate.

## Example 3: Concurrent Web Scraper [ASPIRATIONAL]

> Uses `async`, HTTP client, HTML parsing — none implemented.

```
use http
use html

type Page {
  url: Str
  title: Str
  links: List[Str]
}

fn scrapePage(url: Str) async -> Result[Page, Error] {
  resp := http.get(url).await?
  doc := html.parse(resp.body)

  Page(
    url: url,
    title: doc.select("title")?.text() ?? "untitled",
    links: doc.selectAll("a[href]").map(a => a.attr("href")).collect()
  )
}

fn crawl(startUrl: Str, maxPages: Int) async -> List[Page] {
  visited := Set[Str]()
  mut queue := [startUrl]
  mut pages := List[Page]()

  while queue.notEmpty() and pages.len < maxPages {
    -- Process up to 10 URLs concurrently
    batch := queue.take(10)
    queue = queue.drop(10)

    results := batch
      .filter(url => visited.add(url))  -- add returns false if already present
      .map(url => spawn scrapePage(url))
      .awaitAll()

    for result in results {
      match result {
        Ok(page) => {
          pages.push(page)
          queue.appendAll(page.links)
        }
        Err(e) => log.warn("Failed: {e}")
      }
    }
  }

  pages
}

fn main() async {
  pages := crawl("https://example.com", maxPages: 100).await
  for page in pages {
    print("{page.title} ({page.url}) - {page.links.len} links")
  }
}
```

## Example 4: Data Processing Pipeline [ASPIRATIONAL]

> Uses `deriving(Serialize)`, `DateTime`, `@jsonName` — none implemented.

```
type LogEntry deriving(Serialize) {
  @jsonName("@timestamp")
  timestamp: DateTime
  level: Str
  message: Str
  service: Str
}

type Report {
  totalErrors: Int
  errorsByService: Map[Str, Int]
  topMessages: List[(Str, Int)]
}

fn analyzeLogFile(path: Str) -> Result[Report, Error] {
  errors := readLines(path)
    .map(line => LogEntry.fromJson(line))
    .filterOk()
    .filter(e => e.level == "ERROR")
    .collect()

  Report(
    totalErrors: errors.len,
    errorsByService: errors.countBy(e => e.service),
    topMessages: errors
      .countBy(e => e.message)
      .entries()
      .sortBy(e => -e.value)
      .take(10)
      .map(e => (e.key, e.value))
      .collect()
  )
}

fn main() {
  report := analyzeLogFile("app.log").unwrap("Failed to process log")
  print("Total errors: {report.totalErrors}")
  print("\nErrors by service:")
  for svc, count in report.errorsByService {
    print("  {svc}: {count}")
  }
  print("\nTop error messages:")
  for msg, count in report.topMessages {
    print("  {count:>5}x {msg}")
  }
}
```

## Example 5: One-File Script [ASPIRATIONAL]

> Uses `Dir.read()`, shebang execution, top-level code without `fn main` —
> not all features available in the bootstrap compiler.

The simplest possible program. No main function needed for scripts.

```
#!/usr/bin/env ore run

-- Rename files in current directory to lowercase
for entry in Dir.read(".") {
  if entry.isFile {
    lower := entry.name.toLower()
    if entry.name != lower {
      entry.rename(lower)
      print("Renamed: {entry.name} -> {lower}")
    }
  }
}
```

Scripts can omit `fn main()` — top-level code just runs. Add the shebang
and it works like a shell script, but with type safety and real error
handling.
