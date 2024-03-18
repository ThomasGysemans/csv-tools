use core::fmt;
use std::collections::HashSet;
use std::fs::File;
use std::io::Error;
use std::io::{BufRead, BufReader};
use std::io::ErrorKind;
use std::io::Write;

pub struct CSVFile {
  pub delimiter: char,
  pub columns: Vec<String>,
  pub data: Vec<Vec<String>>,
}

pub struct CSVCoords {
  pub row: usize,
  pub column: usize,
}

impl PartialEq for CSVCoords {
  fn eq(&self, other: &Self) -> bool {
    self.row == other.row && self.column == other.column
  }
}

impl fmt::Display for CSVCoords {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "({}, {})", self.row, self.column)
  }
}

impl fmt::Debug for CSVCoords {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "CSVCoords {{ row: {}, column: {} }}", self.row, self.column)
  }
}

impl fmt::Display for CSVFile {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mut result = String::new();
    for column in &self.columns {
      result.push_str(column);
      result.push(self.delimiter);
    }
    result.pop(); // removes the trailing delimiter
    result.push('\n');

    for row in &self.data {
      for field in row {
        result.push_str(field);
        result.push(self.delimiter);
      }
      result.pop();
      result.push('\n');
    }

    write!(f, "{}", result)
  }
}

impl fmt::Debug for CSVFile {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "CSVFile {{ delimiter: {}, columns: {:?}, data: {:?} }}", self.delimiter, self.columns, self.data)
  }
}

impl CSVFile {
  /// Creates a new CSVFile from a file name and an optional delimiter (a coma by default).
  /// It reads the first line of the file to get the columns and the rest of the file to get the data.
  /// It may return an error if the file doesn't exist or if it can't be read properly.
  pub fn new(file_name: &String, delimiter: &char) -> Result<Self, Error> {
    let file = File::open(&file_name)?;
    let mut lines = BufReader::new(&file).lines();
    let first_line = lines.next().unwrap()?;
    let columns = read_columns(&first_line, delimiter)?;
    let data = read_data(&mut lines, delimiter, columns.len())?;

    Ok(
      Self {
        delimiter: *delimiter,
        columns,
        data
      }
    )
  }

  /// Creates a new CSVFile from the columns and the data.
  /// 
  /// # Example
  /// 
  /// ```
  /// # use csv_tools::CSVFile;
  /// 
  /// let columns = vec!["a".to_string(), "b".to_string(), "c".to_string()];
  /// let rows = vec![
  ///    vec!["1".to_string(), "2".to_string(), "3".to_string()],
  ///    vec!["4".to_string(), "5".to_string(), "6".to_string()],
  ///    vec!["7".to_string(), "8".to_string(), "9".to_string()],
  /// ];
  /// 
  /// let file = CSVFile::build(&columns, &rows, &',').unwrap();
  /// assert_eq!(file.columns, columns);
  /// assert_eq!(file.data, rows);
  /// ```
  pub fn build(columns: &Vec<String>, data: &Vec<Vec<String>>, delimiter: &char) -> Result<Self, Error> {
    for (index, row) in data.iter().enumerate() {
      if columns.len() != row.len() {
        return Err(Error::new(
          ErrorKind::InvalidData,
          format!("Invalid number of fields for row of index {}, {} were given, but expected {}", index, row.len(), columns.len()))
        );
      }
    }

    Ok(
      Self {
        delimiter: *delimiter,
        columns: columns.clone(),
        data: data.clone()
      }
    )
  }

  /// Writes the CSV file to a file.
  pub fn write(&self, filename: &String) -> Result<(), Error> {
    let mut file = File::create(filename)?;
    file.write_all(self.to_string().as_bytes())?;
    Ok(())
  }

  /// Returns the number of columns in the CSV file.
  pub fn len(&self) -> usize {
    self.columns.len()
  }

  /// Returns the number of rows in the CSV file.
  /// It doesn't count the header.
  pub fn count_rows(&self) -> usize {
    self.data.len()
  }

  /// Returns `true` if the CSV file has the given column.
  pub fn has_column(&self, column_name: &String) -> bool {
    self.columns.contains(column_name)
  }

  /// Sets the delimiter of the CSV file.
  pub fn set_delimiter(&mut self, new_delimiter: &char) {
    self.delimiter = *new_delimiter;
  }

  /// Gets the index of a column by its name.
  pub fn get_column_idx(&self, column_name: &String) -> Option<usize> {
    self.columns.iter().position(|c| c == column_name)
  }

  /// Gets a cell at given coordinates.
  /// It returns `None` if the coordinates are out of range.
  /// 
  /// # Example
  /// 
  /// ```
  /// # use csv_tools::{CSVFile, CSVCoords};
  /// 
  /// let columns = vec!["a".to_string(), "b".to_string(), "c".to_string()];
  /// let rows = vec![
  ///    vec!["1".to_string(), "2".to_string(), "3".to_string()],
  ///    vec!["4".to_string(), "5".to_string(), "6".to_string()],
  ///    vec!["7".to_string(), "8".to_string(), "9".to_string()],
  /// ];
  /// 
  /// let file = CSVFile::build(&columns, &rows, &',').unwrap();
  /// 
  /// assert_eq!(file.get_cell(&CSVCoords { row: 0, column: 0 }), Some(&"1".to_string()));
  /// assert_eq!(file.get_cell(&CSVCoords { row: 1, column: 1 }), Some(&"5".to_string()));
  /// assert_eq!(file.get_cell(&CSVCoords { row: 2, column: 2 }), Some(&"9".to_string()));
  /// ```
  pub fn get_cell(&self, coordinates: &CSVCoords) -> Option<&String> {
    self.data.get(coordinates.row)?.get(coordinates.column)
  }

  /// Finds text in the CSV file and returns the coordinates of the cells.
  pub fn find_text(&self, text: &String) -> Vec<CSVCoords> {
    let mut coords: Vec<CSVCoords> = Vec::new();
    for (i, row) in self.data.iter().enumerate() {
      for (j, cell) in row.iter().enumerate() {
        if cell.contains(text) {
          coords.push(CSVCoords { row: i, column: j });
        }
      }
    }

    coords
  }

  /// Checks if the CSV file is valid.
  /// It checks for duplicates in the columns and if the rows have the right length.
  pub fn check_validity(&self) -> bool {
    // Check for duplicates in the columns
    let mut column_names: HashSet<&str> = HashSet::new();
    for column in &self.columns {
      if column_names.contains(column.as_str()) {
        return false;
      }
      column_names.insert(column);
    }

    // Make sure the rows have the right length
    let number_of_columns = self.len();
    for row in &self.data {
      if row.len() != number_of_columns {
        return false;
      }
    }

    true
  }

  /// Fills a column with the given data.
  /// It may return an error if the column doesn't exist or if the length of the data is different from the number of rows.
  pub fn fill_column(&mut self, column_name: &String, data: &Vec<String>) -> Result<(), Error> {
    let column_idx = self.columns.iter().position(|c| c == column_name);

    if column_idx.is_none() {
      Err(Error::new(
        ErrorKind::InvalidData,
        format!("The column {} doesn't exist", column_name))
      )
    } else {
      if data.len() != self.count_rows() {
        Err(Error::new(
          ErrorKind::InvalidData,
          format!("Invalid number of fields, {} were given, but expected {}", data.len(), self.count_rows()))
        )
      } else {
        let column_idx = column_idx.unwrap();
        for (i, row) in self.data.iter_mut().enumerate() {
          row[column_idx] = data[i].clone();
        }
  
        Ok(())
      }
    }
  }

  /// Merges two CSV files together.
  /// It may return an error if a duplicated column is found.
  /// If the number of rows are different, then the rows are extended with empty strings.
  /// 
  /// The other CSVFile instance is supposed to be valid.
  pub fn merge(&mut self, other: &CSVFile) -> Result<(), Error> {
    for column in &other.columns {
      if self.columns.contains(column) {
        return Err(Error::new(
          ErrorKind::InvalidData,
          format!("The column {} already exists", column))
        );
      }
    }

    // If self has less rows than other
    //   -> add rows composed of empty strings to self until the lengths match
    // If self has more rows than other
    //   -> extend the rows of self with empty strings (from the point where the lengths dismatch to the end of the file).
    //      Add as many empty strings as the number of columns in other.
    // Finally:
    //   -> extend the rows of self with the data from other

    let number_of_columns = self.len() + other.len();
    let self_rows = self.count_rows();
    let other_rows = other.count_rows();

    // Add the columns of other to self
    self.columns.extend(other.columns.iter().cloned());

    if self_rows < other_rows {
      for _ in self_rows..other_rows {
        self.data.push(vec![String::new(); number_of_columns]);
      }
    } else if self_rows > other_rows {
      for i in (other_rows + 1)..self_rows {
        self.data[i].extend(vec![String::new(); other.len()].iter().cloned());
      }
    }

    for (i, row) in self.data.iter_mut().enumerate() {
      row.extend(other.data[i].iter().cloned());
    }

    Ok(())
  }

  /// Adds a row to the CSV file.
  /// It may return an error if the number of fields
  /// in the row is different from the number of columns.
  pub fn add_row(&mut self, data: &Vec<String>) -> Result<(), Error> {
    if data.len() != self.len() {
      return Err(Error::new(
        ErrorKind::InvalidData,
        format!("Invalid number of fields, {} were given, but expected {}", data.len(), self.len()))
      );
    }

    self.data.push(data.clone());

    Ok(())
  }

  /// Adds a column to the CSV file.
  /// It may return an error if the column already exists.
  /// It appends an empty string to each row.
  pub fn add_column(&mut self, name: &String) -> Result<(), Error> {
    if self.columns.contains(&name) {
      return Err(Error::new(
        ErrorKind::InvalidData,
        format!("The column {} already exists", name))
      );
    }

    self.columns.push(name.clone());
    for row in &mut self.data {
      row.push(String::new());
    }

    Ok(())
  }

  /// Inserts a column to the CSV file at a specific index.
  /// It may return an error if the column already exists or if the index is out of range.
  /// It also inserts an empty string to each row.
  pub fn insert_column(&mut self, name: &String, column_idx: usize) -> Result<(), Error> {
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

    self.columns.insert(column_idx, name.clone());
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

mod tests;