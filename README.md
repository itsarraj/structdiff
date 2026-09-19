# structdiff

Semantic JSON diff: reports what actually changed by path
(`config.debug: false -> true`), not a line-based text diff that
treats a reordered key or reformatted whitespace as a change.
Python has `jsondiff`, Node has `json-diff`; Rust's `jaq`/`jq`-style
tools are excellent at *querying* JSON but none of them diff two
documents semantically out of the box.

## Usage

```bash
structdiff old.json new.json
```

```
$ structdiff old.json new.json
+ config.features[2] = "metrics"
+ config.timeout = 30
- owners[1] = "bob"
~ config.debug: false -> true
~ version: "1.2.0" -> "1.3.0"
```

`+` added, `-` removed, `~` changed. Exit code `1` if there are any
differences, `0` if none — scriptable the same way `diff` is.

Paths use `a.b.c` for object keys and `a[2]` for array indices. A key
or array index that only exists on one side is reported as a single
entry holding its whole value — adding a nested object shows up as one
`+ path = {...}` line, not a separate line for every field inside it.

## Status: built and verified, directly contrasted against plain-text `diff` on the same input

- **11 unit tests**: identical values produce nothing, reordered
  object keys produce nothing (the core semantic-vs-textual
  distinction this tool exists for), a wholesale-added nested object
  is one entry not a recursive wall of leaf entries, dot notation for
  nested objects, bracket notation for array indices, arrays growing/
  shrinking correctly reported as added/removed at the trailing
  indices, and a type change at the same key (object -> string)
  producing exactly one `Changed` entry instead of crashing or
  producing garbage.
- **Live-verified against a realistic config file change, directly
  compared against plain-text `diff` on the identical input**: a
  6-field config with a reordered top-level key pair (`name`/`version`
  swapped), a nested value flip, a new array element, a removed array
  element, and a new nested key. `structdiff` reported exactly the 5
  real semantic changes with clean paths and **zero noise from the key
  reorder**. Running plain `diff` on the same two files, side by side,
  produced 6 hunks of line-level churn — including phantom "changes"
  purely from the harmless key reorder — that a human has to manually
  untangle to find the same 5 real changes. This side-by-side
  comparison is the whole point of the tool, not an afterthought.
- Separately verified: two genuinely identical files report "no
  differences" with exit `0`.

**Deliberate scope limits, stated plainly**: arrays are compared
**index-by-index**, not with any reordering/LCS detection — inserting
an element at the *start* of an array shifts every later index and
will show up as every element "changing" rather than one clean
insertion, the same limitation `jsondiff`-style positional array
diffing generally has without opting into a more expensive alignment
algorithm. No YAML support (JSON only) — piping YAML through a
JSON converter first is the intended workaround.
