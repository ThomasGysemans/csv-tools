# CSV Tools

A Rust crate to easily read, manipulate and create CSV files, supporting double quotes and escaped characters.

## How to use

See the documentation in [crates.io](https://crates.io/crates/csv-tools) for further information about the individual methods of the crate.

As of now this crate doesn't use any external dependencies.

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

use csv_tools::CSVCoords;

// get the value at specific coordinates
assert_eq!(file.get_cell(&CSVCoords { row: 0, column: 0 }), Some(&"C++".to_string()));
```

## Map your CSV to a data structure

```rust
// continuing with the above example
// ...

#[derive(Debug, PartialEq)]
struct Language {
    name: String,
    level_of_fun: i32,
    level_of_difficulty: i32,
}

// mapped_rows is a vector of Language.
let mapped_rows = file.map_rows(|row: &Vec<String>| {
    Language {
        name: row[0].clone(),
        level_of_fun: row[1].parse().unwrap(),
        level_of_difficulty: row[2].parse().unwrap(),
    }
});

assert_eq!(mapped_rows.len(), 8);
assert_eq!(
    mapped_rows[0],
    Language {
        name: "C++".to_string(),
        level_of_fun: 10,
        level_of_difficulty: 8
    }
);
```

You also have methods such as:

- `find_text`
- `check_validity`
- `trim_end`
- `trim_start`
- `trim`
- `merge` (for merging CSV files)
- ...
