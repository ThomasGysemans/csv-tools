//! # CSV Tools
//!
//! This crate is a collection of utilities to make reading, manipulating and creating CSV files
//! easier. You can open a CSV file, read the text in it and find text, define a delimiter, change
//! it and save the file with the new delimiter, merge CSV files, map the rows with a custom data
//! structure, etc.
//!
//! Note that quotes are supported with this crate. Meaning that if a cell is surrounded by double
//! quotes, it will count as one unique value instead of multiple values if it were to contain the
//! delimiter.
//!
//! For example, assuming the delimiter is a comma:
//!
//! ```csv
//! name,pseudo,age
//! Thomas,"The Svelter",20
//! Yoshiip,"The best, and only, Godoter",99
//! ```
//!
//! The second row contains 3 values. However, without the quotes it would have been parsed as 5
//! different values: `"Yoshiip"`, `"The best"`, `" and only"`, `" Godoter", 99` since it contains
//! the delimiter.
//!
//! By default, a row without double quotes will be parsed using a simple built-in method
//! (`split`), which is slightly more performant since less calculations are needed to find and
//! locate the right ending of a string.
//!
//! Escape characters are allowed, meaning that a string can contain `\"`.

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fs::File;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Write;
use std::io::{BufRead, BufReader};

/// A simple data structure for holding the raw string data of a CSV file.
pub struct CSVFile {
    pub delimiter: char,
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

/// A simple data structure for identifying the position of a cell within a CSV file.
#[derive(PartialEq)]
pub struct CSVCoords {
    pub row: usize,
    pub column: usize,
}

impl fmt::Display for CSVCoords {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.row, self.column)
    }
}

impl fmt::Debug for CSVCoords {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "CSVCoords {{ row: {}, column: {} }}",
            self.row, self.column
        )
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

        for row in &self.rows {
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
        write!(
            f,
            "CSVFile {{ delimiter: {}, columns: {:?}, rows: {:?} }}",
            self.delimiter, self.columns, self.rows
        )
    }
}

impl CSVFile {
    /// Creates a new CSVFile from a file name and an optional delimiter (a comma by default).
    /// It reads the first line of the file to get the columns and the rest of the file to get the data.
    /// It may return an error if the file doesn't exist or if it can't be read properly.
    pub fn new(file_name: &String, delimiter: &char) -> Result<Self, Error> {
        let file = File::open(&file_name)?;
        let mut lines = BufReader::new(&file).lines();
        let first_line = lines.next().unwrap()?;
        let columns = read_columns(&first_line, delimiter)?;
        let rows = read_rows(&mut lines, delimiter, columns.len())?;

        Ok(Self {
            delimiter: *delimiter,
            columns,
            rows,
        })
    }

    /// Creates a new CSVFile from the columns and the rows.
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
    /// assert_eq!(file.rows, rows);
    /// ```
    pub fn build(
        columns: &Vec<String>,
        rows: &Vec<Vec<String>>,
        delimiter: &char,
    ) -> Result<Self, Error> {
        for (index, row) in rows.iter().enumerate() {
            if columns.len() != row.len() {
                return Err(Error::new(
          ErrorKind::InvalidData,
          format!("Invalid number of fields for row of index {}, {} were given, but expected {}", index, row.len(), columns.len()))
        );
            }
        }

        Ok(Self {
            delimiter: *delimiter,
            columns: columns.clone(),
            rows: rows.clone(),
        })
    }

    /// Maps the rows of the CSV file to a type `T` using a callback function `F` called on each row.
    ///
    /// # Example
    ///
    /// ```
    /// # use csv_tools::CSVFile;
    ///
    /// #[derive(Debug, PartialEq)]
    /// struct MyData {
    ///   a: u32,
    ///   b: u32,
    ///   c: u32
    /// }
    ///
    /// let columns = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    /// let rows = vec![
    ///    vec!["1".to_string(), "2".to_string(), "3".to_string()],
    ///    vec!["4".to_string(), "5".to_string(), "6".to_string()],
    ///    vec!["7".to_string(), "8".to_string(), "9".to_string()],
    /// ];
    ///
    /// let csv_file = CSVFile::build(&columns, &rows, &',').unwrap();
    /// let result = csv_file.map_rows(|row: &Vec<String>| {
    ///   MyData {
    ///     a: row[0].parse().unwrap(),
    ///     b: row[1].parse().unwrap(),
    ///     c: row[2].parse().unwrap()
    ///   }
    /// });
    ///
    /// assert_eq!(result.len(), 3);
    /// assert_eq!(result[0], MyData { a: 1, b: 2, c: 3 });
    /// assert_eq!(result[1], MyData { a: 4, b: 5, c: 6 });
    /// assert_eq!(result[2], MyData { a: 7, b: 8, c: 9 });
    /// ```
    pub fn map_rows<F, T>(&self, f: F) -> Vec<T>
    where
        F: Fn(&Vec<String>) -> T,
    {
        self.rows.iter().map(f).collect()
    }

    /// Maps the columns of the CSV file to a HashMap.
    fn map_columns<T>(&self) -> HashMap<String, Vec<T>> {
        let mut map = HashMap::new();
        for column in &self.columns {
            map.insert(column.clone(), Vec::new());
        }

        map
    }

    /// Maps the columns of the CSV file. The keys are column names and the associated value is a vector of type `T`.
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
    /// let csv_file = CSVFile::build(&columns, &rows, &',').unwrap();
    /// let result = csv_file.to_map(|row: &String| row.parse::<u32>().unwrap());
    ///
    /// assert_eq!(result.len(), 3);
    /// assert_eq!(result.get(&String::from("a")).unwrap(), &vec![1, 4, 7]);
    /// assert_eq!(result.get(&String::from("b")).unwrap(), &vec![2, 5, 8]);
    /// assert_eq!(result.get(&String::from("c")).unwrap(), &vec![3, 6, 9]);
    /// ```
    pub fn to_map<F, T>(&self, f: F) -> HashMap<String, Vec<T>>
    where
        F: Fn(&String) -> T,
    {
        let mut map: HashMap<String, Vec<T>> = self.map_columns();
        for row in &self.rows {
            for (i, field) in row.iter().enumerate() {
                map.get_mut(&self.columns[i]).unwrap().push(f(field));
            }
        }

        map
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
        self.rows.len()
    }

    /// Returns `true` if the CSV file has the given column.
    pub fn has_column(&self, column_name: &String) -> bool {
        self.columns.contains(column_name)
    }

    /// Returns `true` if the CSV file has no row.
    pub fn has_no_rows(&self) -> bool {
        self.rows.is_empty()
    }

    /// Returns `true` if the CSV file has no column.
    pub fn has_no_columns(&self) -> bool {
        self.columns.is_empty()
    }

    /// Returns `true` if the CSV file is empty,
    /// meaning it doesn't have any column and any row.
    pub fn empty(&self) -> bool {
        self.has_no_rows() && self.has_no_columns()
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
        self.rows.get(coordinates.row)?.get(coordinates.column)
    }

    /// Finds text in the CSV file and returns the coordinates of the cells.
    pub fn find_text(&self, text: &String) -> Vec<CSVCoords> {
        let mut coords: Vec<CSVCoords> = Vec::new();
        for (i, row) in self.rows.iter().enumerate() {
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
        for row in &self.rows {
            if row.len() != number_of_columns {
                return false;
            }
        }

        true
    }

    /// Fills a column with the given data.
    /// It may return an error if the column doesn't exist
    /// or if the length of the data is different from the number of rows.
    pub fn fill_column(&mut self, column_name: &String, data: &Vec<String>) -> Result<(), Error> {
        let column_idx = self.columns.iter().position(|c| c == column_name);

        if column_idx.is_none() {
            Err(Error::new(
                ErrorKind::InvalidData,
                format!("The column {} doesn't exist", column_name),
            ))
        } else {
            if data.len() != self.count_rows() {
                Err(Error::new(
                    ErrorKind::InvalidData,
                    format!(
                        "Invalid number of fields, {} were given, but expected {}",
                        data.len(),
                        self.count_rows()
                    ),
                ))
            } else {
                let column_idx = column_idx.unwrap();
                for (i, row) in self.rows.iter_mut().enumerate() {
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
                    format!("The column {} already exists", column),
                ));
            }
        }

        // If self has less rows than other
        //   -> add rows composed of empty strings to self until the lengths match
        // If self has more rows than other
        //   -> extend the rows of self with empty strings (from the point where the lengths dismatch to the end of the file).
        //      Add as many empty strings as the number of columns in other.
        // Finally:
        //   -> extend the rows of self with the data from other

        let initial_self_len = self.len();
        let self_rows = self.count_rows();
        let other_rows = other.count_rows();

        // Add the columns of other to self
        self.columns.extend(other.columns.iter().cloned());

        if self_rows < other_rows {
            for _ in self_rows..other_rows {
                self.rows.push(vec![String::new(); initial_self_len]);
            }
        } else if self_rows > other_rows {
            for i in other_rows..self_rows {
                self.rows[i].extend(vec![String::new(); other.len()].iter().cloned());
            }
        }

        for i in 0..other_rows {
            self.rows[i].extend(other.rows[i].iter().cloned());
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
                format!(
                    "Invalid number of fields, {} were given, but expected {}",
                    data.len(),
                    self.len()
                ),
            ));
        }

        self.rows.push(data.clone());

        Ok(())
    }

    /// Adds a column to the CSV file.
    /// It may return an error if the column already exists.
    /// It appends an empty string to each row.
    pub fn add_column(&mut self, name: &String) -> Result<(), Error> {
        if self.columns.contains(&name) {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("The column {} already exists", name),
            ));
        }

        self.columns.push(name.clone());
        for row in &mut self.rows {
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
                format!("The column index {} is out of range", column_idx),
            ));
        }

        if self.columns.contains(&name) {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("The column {} already exists", name),
            ));
        }

        self.columns.insert(column_idx, name.clone());
        for row in &mut self.rows {
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
                format!("The column index {} is out of range", column_idx),
            ));
        }

        self.columns.remove(column_idx);
        for row in &mut self.rows {
            row.remove(column_idx);
        }

        Ok(())
    }

    /// Removes a row from the CSV file.
    /// It may return an error if the row index is out of range.
    pub fn remove_row(&mut self, row_idx: usize) -> Result<(), Error> {
        if row_idx >= self.rows.len() {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("The row index {} is out of range", row_idx),
            ));
        }

        self.rows.remove(row_idx);

        Ok(())
    }

    /// Removes all the rows that are composed of empty strings only,
    /// starting at the very end and stopping as soon as a non-empty row is found.
    ///
    /// If no empty row is found, then nothing happens.
    pub fn trim_end(&mut self) {
        let mut i = self.rows.len() - 1;
        loop {
            if self.rows[i].iter().all(|s| s.is_empty()) {
                self.rows.remove(i);
                if i == 0 {
                    break;
                } else {
                    i -= 1;
                }
            } else {
                break;
            }
        }
    }

    /// Removes all the rows that are composed of empty strings only,
    /// starting at the very beginning and stopping as soon as a non-empty row is found.
    ///
    /// If no empty row is found, then nothing happens.
    pub fn trim_start(&mut self) {
        let mut to_remove: Vec<usize> = Vec::new();
        let mut i = 0;
        while i < self.rows.len() {
            if self.rows[i].iter().all(|s| s.is_empty()) {
                to_remove.push(i);
                i += 1;
            } else {
                break;
            }
        }
        for i in to_remove.into_iter().rev() {
            self.rows.remove(i);
        }
    }

    /// Removes all the rows that are composed of empty strings only at the beginning and at the end.
    pub fn trim(&mut self) {
        self.trim_start();
        self.trim_end();
    }

    /// Removes all the empty lines from the CSV file.
    pub fn remove_empty_lines(&mut self) {
        self.rows.retain(|row| !row.iter().all(|s| s.is_empty()));
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
pub(crate) fn parse_line(
    line: &String,
    delimiter: &char,
    number_of_fields: Option<u32>,
) -> Result<Vec<String>, Error> {
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
        return Err(Error::new(
            ErrorKind::InvalidData,
            "Invalid escape sequence",
        ));
    }

    // Push the last field
    fields.push(current_field);

    Ok(fields)
}

/// Splits the line into a vector of strings using the delimiter.
/// Contrary to [parse_line](`#parse_line`), this function uses the split method.
pub(crate) fn split_line(line: &String, delimiter: &char) -> Vec<String> {
    line.split(*delimiter).map(|s| s.to_string()).collect()
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
        Ok(split_line(line, delimiter))
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
pub(crate) fn read_rows(
    lines: &mut std::io::Lines<BufReader<&File>>,
    delimiter: &char,
    number_of_fields: usize,
) -> Result<Vec<Vec<String>>, Error> {
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
