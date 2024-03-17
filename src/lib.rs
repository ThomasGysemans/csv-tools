use std::fs::File;
use std::io::Error;
use std::io::{BufRead, BufReader};
use std::io::ErrorKind;

pub struct CSVFile {
  pub delimiter: char,
  pub columns: Vec<String>,
  pub data: Vec<Vec<String>>,
}

impl CSVFile {
  /// Creates a new CSVFile from a file name and an optional delimiter (a coma by default).
  /// It reads the first line of the file to get the columns and the rest of the file to get the data.
  /// It may return an error if the file doesn't exist or if it can't be read properly.
  pub fn new(file_name: &String, delimiter: &Option<char>) -> Result<Self, Error> {
    let actual_delimiter = delimiter.unwrap_or(',');
    let file = File::open(&file_name)?;
    let mut lines = BufReader::new(&file).lines();
    let first_line = lines.next().unwrap()?;
    let columns = read_columns(&first_line, &actual_delimiter)?;
    let data = read_data(&mut lines, &actual_delimiter, columns.len())?;

    Ok(
      Self {
        delimiter: actual_delimiter,
        columns,
        data
      }
    )
  }

  /// Returns the number of columns in the CSV file.
  pub fn len(&self) -> usize {
    self.columns.len()
  }

  /// Adds a row to the CSV file.
  /// It may return an error if the number of fields
  /// in the row is different from the number of columns.
  pub fn add_row(&mut self, data: Vec<String>) -> Result<(), Error> {
    if data.len() != self.len() {
      return Err(Error::new(
        ErrorKind::InvalidData,
        format!("Invalid number of fields, {} were given, but expected {}", data.len(), self.len()))
      );
    }

    self.data.push(data);

    Ok(())
  }

  /// Adds a column to the CSV file.
  /// It may return an error if the column already exists.
  /// It appends an empty string to each row.
  pub fn add_column(&mut self, name: String) -> Result<(), Error> {
    if self.columns.contains(&name) {
      return Err(Error::new(
        ErrorKind::InvalidData,
        format!("The column {} already exists", name))
      );
    }

    self.columns.push(name);
    for row in &mut self.data {
      row.push(String::new());
    }

    Ok(())
  }

  /// Inserts a column to the CSV file at a specific index.
  /// It may return an error if the column already exists or if the index is out of range.
  /// It also inserts an empty string to each row.
  pub fn insert_column(&mut self, name: String, column_idx: usize) -> Result<(), Error> {
    if column_idx > self.len() {
      return Err(Error::new(
        ErrorKind::InvalidData,
        format!("The column index {} is out of range", column_idx))
      );
    }

    if self.columns.contains(&name) {
      return Err(Error::new(
        ErrorKind::InvalidData,
        format!("The column {} already exists", name))
      );
    }

    self.columns.insert(column_idx, name);
    for row in &mut self.data {
      row.insert(column_idx, String::new());
    }

    Ok(())
  }

  /// Removes a column from the CSV file.
  /// It may return an error if the column index is out of range.
  pub fn remove_column(&mut self, column_idx: usize) -> Result<(), Error> {
    if column_idx >= self.len() {
      return Err(Error::new(
        ErrorKind::InvalidData,
        format!("The column index {} is out of range", column_idx))
      );
    }

    self.columns.remove(column_idx);
    for row in &mut self.data {
      row.remove(column_idx);
    }

    Ok(())
  }

  /// Removes a row from the CSV file.
  /// It may return an error if the row index is out of range.
  pub fn remove_row(&mut self, row_idx: usize) -> Result<(), Error> {
    if row_idx >= self.data.len() {
      return Err(Error::new(
        ErrorKind::InvalidData,
        format!("The row index {} is out of range", row_idx))
      );
    }

    self.data.remove(row_idx);

    Ok(())
  }
}

/// Parses the line into a vector of strings.
/// It does so by reading the line character by character.
/// If the character is not the delimiter, it appends it to the current field.
/// If the character is the delimiter, it appends the current field to the vector and starts a new field.
/// 
/// The point of this function is to avoid using the split method, as it would ignore quotes.
/// Indeed, if a cell is a string we want to ignore the delimiters inside it.
/// 
/// The "number_of_fields" parameter is used to pre-allocate the vectors.
/// This is useful when we know the number of fields in advance.
pub(crate) fn parse_line(line: &String, delimiter: &char, number_of_fields: Option<u32>) -> Result<Vec<String>, Error> {
  let mut fields: Vec<String> = match number_of_fields {
    Some(n) => Vec::with_capacity(n as usize),
    None => Vec::new(),
  };

  let mut chars = line.chars();
  let mut current_field = String::new();
  let mut is_in_quote = false;
  let mut is_escaped = false;

  while let Some(c) = chars.next() {
    if c == '\\' {
      if is_escaped {
        current_field.push(c);
      }
      is_escaped = !is_escaped;
    } else {
      if c == '"' {
        if !is_escaped {
          if is_in_quote {
            fields.push(current_field);
            current_field = String::new();
            // skip the next character because it should be
            // the delimiter (or the end of the line)
            chars.next();
          }
          is_in_quote = !is_in_quote;
        } else {
          current_field.push(c);
        }
      } else {
        if c == *delimiter && !is_in_quote {
          fields.push(current_field);
          current_field = String::new();
        } else {
          current_field.push(c);
        }
      }
      // If the character immediately following a blackslash
      // isn't another backslash, then make sure to be unescaped.
      is_escaped = false;
    }
  }

  if is_escaped || is_in_quote {
    return Err(Error::new(ErrorKind::InvalidData, "Invalid escape sequence"));
  }

  // Push the last field
  fields.push(current_field);

  Ok(fields)
}

/// Splits the line into a vector of strings using the delimiter.
/// Contrary to [parse_line](`#parse_line`), this function uses the split method.
pub(crate) fn split_line(line: &String, delimiter: &char) -> Vec<String> {
  line
    .split(*delimiter)
    .map(|s| s.to_string())
    .collect()
}

/// Reads the columns of the CSV file.
/// If the line contains quotes (double quotes), it uses the [parse_line](`#parse_line`) function.
/// Otherwise, it uses the [split_line](`#split_line`) function.
/// 
/// It returns a Result because it can fail if the line,
/// contains an invalid escape sequence or an unclosed quote.
pub(crate) fn read_columns(line: &String, delimiter: &char) -> Result<Vec<String>, Error> {
  if line.contains('"') {
    parse_line(line, delimiter, None)
  } else {
    Ok(
      split_line(line, delimiter)
    )
  }
}

/// Reads the data of the CSV file.
/// It reads the lines of the file and uses the [parse_line](`#parse_line`) function if the line contains double quotes.
/// Otherwise, it uses the [split_line](`#split_line`) function.
/// 
/// It returns a Result because it can fail if the line,
/// contains an invalid escape sequence or an unclosed quote.
/// 
/// The "number_of_fields" parameter is used to pre-allocate the vectors.
/// This is useful when we know the number of fields in advance.
pub(crate) fn read_data(lines: &mut std::io::Lines<BufReader<&File>>, delimiter: &char, number_of_fields: usize) -> Result<Vec<Vec<String>>, Error> {
  let mut data: Vec<Vec<String>> = Vec::new();

  for line in lines {
    let line = line?;
    let fields: Vec<String>;
    if line.contains('"') {
      fields = parse_line(&line, delimiter, Some(number_of_fields as u32))?;
    } else {
      fields = split_line(&line, delimiter);
    }
    data.push(fields);
  }

  Ok(data)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_line() {
    let line = r"a,b,c".to_string();
    let result = parse_line(&line, &',', None).unwrap();
    assert_eq!(result, vec!["a", "b", "c"]);

    let line = r#"a,"Hello, World!",c"#.to_string();
    let result = parse_line(&line, &',', None).unwrap();
    assert_eq!(result, vec!["a", "Hello, World!", "c"]);

    let line = r#"a,"Hello, \\World!",c"#.to_string();
    let result = parse_line(&line, &',', None).unwrap();
    assert_eq!(result, vec!["a", r"Hello, \World!", "c"]);

    let line = r#"a,"Hello, \\\\World!",c"#.to_string();
    let result = parse_line(&line, &',', None).unwrap();
    assert_eq!(result, vec!["a", r"Hello, \\World!", "c"]);

    let line = r#"a,"Hello, \\\World!",c"#.to_string();
    let result = parse_line(&line, &',', None).unwrap();
    assert_eq!(result, vec!["a", r"Hello, \World!", "c"]);

    let line = r#"a,"Hello, \"World!",c"#.to_string();
    let result = parse_line(&line, &',', None).unwrap();
    assert_eq!(result, vec!["a", r#"Hello, "World!"#, "c"]);

    // Unclosed quote
    let line = r#"a,"Hello, World!,c"#.to_string();
    let result = parse_line(&line, &',', None);
    assert!(result.is_err());

    // Invalid escape sequence
    let line = r#"a,"Hello, World!",c\"#.to_string();
    let result = parse_line(&line, &',', None);
    assert!(result.is_err());
  }

  #[test]
  fn test_split_line() {
    let line = r"a,b,c".to_string();
    let result = split_line(&line, &',');
    assert_eq!(result, vec!["a", "b", "c"]);

    let line = r"a,'Hello, World!',c".to_string();
    let result = split_line(&line, &',');
    assert_eq!(result, vec!["a", "'Hello", " World!'", "c"]);
  }

  #[test]
  fn test_read_columns() {
    let line = r"a,b,c".to_string();
    let result = read_columns(&line, &',').unwrap();
    assert_eq!(result, vec!["a", "b", "c"]);

    let line = r#"a,"Hello, World!",c"#.to_string();
    let result = read_columns(&line, &',').unwrap();
    assert_eq!(result, vec!["a", "Hello, World!", "c"]);
  }
}