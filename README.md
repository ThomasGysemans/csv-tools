# CSV Tools

A Rust crate to easily read and manipulate CSV files.

## How to use

See the documentation in [crates.io](https://crates.io) for further information about the individual methods of the crate.

As of now this crate doesn't use any external dependencies, but it will likely change.

## Simple overview

Here a basic overview with the following example (`langs.csv`):

```csv
language,level_of_fun,level_of_difficulty
C++,10,8
Rust,10,9
JavaScript,9,1
TypeScript,10,1
Java,0,2
HTML,10,-1
GDScript,10,1
Lua,7,1
```

Read the file:

```rust
use csv_tools::CSVFile;

let filename = String::from("langs.csv");
let file = CSVFile::new(&filename, &',')?;

assert_eq!(file.columns, vec![
  "language".to_string(),
  "level_of_fun".to_string(),
  "level_of_difficulty".to_string()
]);

assert_eq!(file.rows, vec![
  vec!["C++".to_string(),        "10".to_string(),  "8".to_string()],
  vec!["Rust".to_string(),       "10".to_string(),  "9".to_string()],
  vec!["JavaScript".to_string(), "9".to_string(),   "1".to_string()],
  vec!["TypeScript".to_string(), "10".to_string(),  "1".to_string()],
  vec!["Java".to_string(),       "0".to_string(),   "2".to_string()],
  vec!["HTML".to_string(),       "10".to_string(), "-1".to_string()],
  vec!["GDScript".to_string(),   "10".to_string() , "1".to_string()],
  vec!["Lua".to_string(),        "7".to_string(),   "1".to_string()],
]);
```

A lot of utility methods are here to help you manipulate the data more easily:

```rust
// continuing with the above example
// ...

use csv_tools::CSVFilter;

// get the value at specific coordinates
assert_eq!(file.get_cell(&CSVCoords { row: 0, column: 0 }), Some(&"C++".to_string()));
```

You also have methods such as:

- `find_text`
- `check_validity`
- `trim_end`
- `trim_start`
- `trim`
- `merge`
- ...