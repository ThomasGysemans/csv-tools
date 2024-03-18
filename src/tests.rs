#[cfg(test)]
mod tests {
  use crate::*;
  use std::fs;
  use std::fs::File;
  use std::io::Read;

  fn get_fake_columns() -> Vec<String> {
    vec!["a".to_string(), "b".to_string(), "c".to_string()]
  }

  fn get_fake_rows() -> Vec<Vec<String>> {
    vec![
      vec!["1".to_string(), "2".to_string(), "3".to_string()],
      vec!["4".to_string(), "5".to_string(), "6".to_string()],
      vec!["7".to_string(), "8".to_string(), "9".to_string()],
    ]
  }

  fn empty_string() -> String {
    String::new()
  }

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

  #[test]
  fn test_build_csv_file() {
    let columns = get_fake_columns();
    let data = get_fake_rows();
    let csv_file = CSVFile::build(&columns, &data, &',').unwrap();
    assert_eq!(csv_file.columns, columns);
    assert_eq!(csv_file.data, data);
  }

  #[test]
  fn test_try_build_invalid_csv_file_with_missing_column() {
    let columns = vec!["a".to_string(), "b".to_string()];
    let data = vec![
      vec!["1".to_string(), "2".to_string(), "3".to_string()],
      vec!["4".to_string(), "5".to_string(), "6".to_string()],
      vec!["7".to_string(), "8".to_string(), "9".to_string()],
    ];
    let result = CSVFile::build(&columns, &data, &',');
    assert!(result.is_err());
  }

  #[test]
  fn test_try_build_invalid_csv_file_with_missing_row() {
    let columns = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let data = vec![
      vec!["1".to_string(), "2".to_string(), "3".to_string()],
      vec!["4".to_string(), "5".to_string()],
      vec!["7".to_string(), "8".to_string(), "9".to_string()],
    ];
    let result = CSVFile::build(&columns, &data, &',');
    assert!(result.is_err());
  }

  #[test]
  fn test_len() {
    let columns = get_fake_columns();
    let data = get_fake_rows();
    let csv_file = CSVFile::build(&columns, &data, &',').unwrap();
    assert_eq!(csv_file.len(), 3);
  }

  #[test]
  fn test_count_rows() {
    let columns = get_fake_columns();
    let data = get_fake_rows();
    let csv_file = CSVFile::build(&columns, &data, &',').unwrap();
    assert_eq!(csv_file.count_rows(), 3);
  }

  #[test]
  fn test_has_column() {
    let columns = get_fake_columns();
    let data = get_fake_rows();
    let csv_file = CSVFile::build(&columns, &data, &',').unwrap();
    assert!(csv_file.has_column(&"a".to_string()));
    assert!(csv_file.has_column(&"b".to_string()));
    assert!(csv_file.has_column(&"c".to_string()));
    assert!(!csv_file.has_column(&"d".to_string()));
  }

  #[test]
  fn test_has_no_column() {
    let columns = get_fake_columns();
    let data = get_fake_rows();
    let mut csv_file = CSVFile::build(&columns, &data, &',').unwrap();
    csv_file.columns.clear();
    assert!(csv_file.has_no_columns());
  }

  #[test]
  fn test_has_no_row() {
    let columns = get_fake_columns();
    let data = get_fake_rows();
    let mut csv_file = CSVFile::build(&columns, &data, &',').unwrap();
    csv_file.data.clear();
    assert!(csv_file.has_no_rows());
  }

  #[test]
  fn test_set_delimiter() {
    let columns = get_fake_columns();
    let data = get_fake_rows();
    let mut csv_file = CSVFile::build(&columns, &data, &',').unwrap();
    assert_eq!(csv_file.delimiter, ',');
    csv_file.set_delimiter(&';');
    assert_eq!(csv_file.delimiter, ';');
  }

  #[test]
  fn test_fill_column() {
    let columns = get_fake_columns();
    let data = get_fake_rows();
    let mut csv_file = CSVFile::build(&columns, &data, &',').unwrap();
    let new_data = vec!["10".to_string(), "11".to_string(), "12".to_string()];
    assert_eq!(csv_file.data[0][1], "2");
    assert_eq!(csv_file.data[1][1], "5");
    assert_eq!(csv_file.data[2][1], "8");
    csv_file.fill_column(&"b".to_string(), &new_data).unwrap();
    assert_eq!(csv_file.data[0][1], "10");
    assert_eq!(csv_file.data[1][1], "11");
    assert_eq!(csv_file.data[2][1], "12");
  }

  #[test]
  fn test_check_validity_on_valid_csv() {
    let columns = get_fake_columns();
    let data = get_fake_rows();
    let csv_file = CSVFile::build(&columns, &data, &',').unwrap();
    assert!(csv_file.check_validity());
  }

  #[test]
  fn test_check_validity_on_invalid_csv() {
    let columns = get_fake_columns();
    let data = get_fake_rows();
    let mut csv_file = CSVFile::build(&columns, &data, &',').unwrap();
    csv_file.columns.remove(1);
    assert!(!csv_file.check_validity());
  }

  #[test]
  fn test_check_validity_with_duplicated_columns() {
    let columns = get_fake_columns();
    let data = get_fake_rows();
    let mut csv_file = CSVFile::build(&columns, &data, &',').unwrap();
    csv_file.columns[0] = "b".to_string();
    assert!(!csv_file.check_validity());
  }

  #[test]
  fn test_check_validity_with_missing_row() {
    let columns = get_fake_columns();
    let data = get_fake_rows();
    let mut csv_file = CSVFile::build(&columns, &data, &',').unwrap();
    csv_file.data[0].remove(1);
    assert!(!csv_file.check_validity());
  }

  #[test]
  fn test_add_row() {
    let columns = get_fake_columns();
    let data = get_fake_rows();
    let mut csv_file = CSVFile::build(&columns, &data, &',').unwrap();
    let new_row = vec!["10".to_string(), "11".to_string(), "12".to_string()];
    csv_file.add_row(&new_row).unwrap();
    assert_eq!(csv_file.data[3], new_row);
  }

  #[test]
  fn test_add_row_with_invalid_data() {
    let columns = get_fake_columns();
    let data = get_fake_rows();
    let mut csv_file = CSVFile::build(&columns, &data, &',').unwrap();
    let new_row = vec!["10".to_string(), "11".to_string(), "12".to_string(), "13".to_string()];
    assert!(csv_file.add_row(&new_row).is_err());
  }

  #[test]
  fn test_add_column() {
    let columns = get_fake_columns();
    let data = get_fake_rows();
    let mut csv_file = CSVFile::build(&columns, &data, &',').unwrap();
    csv_file.add_column(&"d".to_string()).unwrap();
    assert_eq!(csv_file.columns[3], "d");
    assert_eq!(csv_file.data[0][3].len(), 0);
    assert_eq!(csv_file.data[1][3].len(), 0);
    assert_eq!(csv_file.data[2][3].len(), 0);
  }

  #[test]
  fn test_add_invalid_column() {
    let columns = get_fake_columns();
    let data = get_fake_rows();
    let mut csv_file = CSVFile::build(&columns, &data, &',').unwrap();
    assert!(csv_file.add_column(&"a".to_string()).is_err()); // it already exists
  }

  #[test]
  fn test_insert_column() {
    let columns = get_fake_columns();
    let data = get_fake_rows();
    let mut csv_file = CSVFile::build(&columns, &data, &',').unwrap();
    assert_eq!(csv_file.columns[0], "a");
    assert_eq!(csv_file.columns.len(), 3);
    csv_file.insert_column(&"d".to_string(), 0).unwrap();
    assert_eq!(csv_file.columns[0], "d");
    assert_eq!(csv_file.columns.len(), 4);
    assert_eq!(csv_file.data.len(), 3); // there is still 3 rows, but each row got extended by 1
    assert_eq!(csv_file.data[0].len(), 4);
    assert_eq!(csv_file.data[1].len(), 4);
    assert_eq!(csv_file.data[2].len(), 4);
    assert_eq!(csv_file.data[0][0].len(), 0); // empty strings
    assert_eq!(csv_file.data[1][0].len(), 0);
    assert_eq!(csv_file.data[2][0].len(), 0);
  }

  #[test]
  fn test_get_column_idx_by_name() {
    let columns = get_fake_columns();
    let data = get_fake_rows();
    let csv_file = CSVFile::build(&columns, &data, &',').unwrap();
    assert_eq!(csv_file.get_column_idx(&"a".to_string()).unwrap(), 0);
    assert_eq!(csv_file.get_column_idx(&"b".to_string()).unwrap(), 1);
    assert_eq!(csv_file.get_column_idx(&"c".to_string()).unwrap(), 2);
    assert!(csv_file.get_column_idx(&"d".to_string()).is_none());
  }

  #[test]
  fn test_remove_column() {
    let columns = get_fake_columns();
    let data = get_fake_rows();
    let mut csv_file = CSVFile::build(&columns, &data, &',').unwrap();
    assert_eq!(csv_file.columns[0], "a");
    assert_eq!(csv_file.columns.len(), 3);
    csv_file.remove_column(0).unwrap();
    assert_eq!(csv_file.columns[0], "b");
    assert_eq!(csv_file.columns.len(), 2);
    assert_eq!(csv_file.data.len(), 3);
    assert_eq!(csv_file.data[0].len(), 2);
    assert_eq!(csv_file.data[1].len(), 2);
    assert_eq!(csv_file.data[2].len(), 2);
  }

  #[test]
  fn test_remove_row() {
    let columns = get_fake_columns();
    let data = get_fake_rows();
    let mut csv_file = CSVFile::build(&columns, &data, &',').unwrap();
    assert_eq!(csv_file.data.len(), 3);
    csv_file.remove_row(0).unwrap();
    assert_eq!(csv_file.data.len(), 2);
  }

  #[test]
  fn test_write() {
    let columns = get_fake_columns();
    let data = get_fake_rows();
    let csv_file = CSVFile::build(&columns, &data, &',').unwrap();
    let target_filename = String::from("test.csv");
    csv_file.write(&target_filename).unwrap();
    let mut file = File::open(target_filename).unwrap();
    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();
    assert_eq!(contents, "a,b,c\n1,2,3\n4,5,6\n7,8,9\n");
    fs::remove_file("test.csv").unwrap();
  }

  #[test]
  fn test_get_cell() {
    let columns = get_fake_columns();
    let data = get_fake_rows();
    let csv_file = CSVFile::build(&columns, &data, &',').unwrap();
    assert_eq!(csv_file.get_cell(&CSVCoords { row: 0, column: 0 }).unwrap(), "1");
    assert_eq!(csv_file.get_cell(&CSVCoords { row: 1, column: 1 }).unwrap(), "5");
    assert_eq!(csv_file.get_cell(&CSVCoords { row: 2, column: 2 }).unwrap(), "9");
  }

  #[test]
  fn test_find_text() {
    let columns = get_fake_columns();
    let data = get_fake_rows();
    let csv_file = CSVFile::build(&columns, &data, &',').unwrap();
    let result = csv_file.find_text(&"5".to_string());
    assert_eq!(result.len(), 1);
    assert_eq!(result[0], CSVCoords { row: 1, column: 1 });
  }

  #[test]
  fn test_trim_end() {
    let columns = get_fake_columns();
    let data = vec![
      vec!["1".to_string(), "2".to_string(), "3".to_string()],
      vec!["4".to_string(), "5".to_string(), "6".to_string()],
      vec!["7".to_string(), "8".to_string(), "9".to_string()],
      vec![empty_string(), empty_string(), empty_string()],
      vec![empty_string(), "8".to_string(), empty_string()],
      vec![empty_string(), empty_string(), empty_string()],
      vec![empty_string(), empty_string(), empty_string()],
    ];
    let mut csv_file = CSVFile::build(&columns, &data, &',').unwrap();
    assert_eq!(csv_file.count_rows(), 7);
    csv_file.trim_end();
    assert_eq!(csv_file.count_rows(), 5);
  }

  #[test]
  fn test_trim_end_with_a_file_of_one_nonempty_line() {
    let columns = get_fake_columns();
    let data = vec![vec!["1".to_string(), "2".to_string(), "3".to_string()]];
    let mut csv_file = CSVFile::build(&columns, &data, &',').unwrap();
    assert_eq!(csv_file.count_rows(), 1);
    csv_file.trim_end();
    assert_eq!(csv_file.count_rows(), 1);
  }

  #[test]
  fn test_trim_end_with_a_file_of_one_empty_line() {
    let columns = get_fake_columns();
    let data = vec![vec![empty_string(), empty_string(), empty_string()]];
    let mut csv_file = CSVFile::build(&columns, &data, &',').unwrap();
    assert_eq!(csv_file.count_rows(), 1);
    csv_file.trim_end();
    assert_eq!(csv_file.count_rows(), 0);
    assert!(csv_file.has_no_rows())
  }
}