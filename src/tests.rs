#[cfg(test)]
mod tests {
  use crate::*;

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
    let csv_file = CSVFile::build(&columns, &data, &None).unwrap();
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
    let result = CSVFile::build(&columns, &data, &None);
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
    let result = CSVFile::build(&columns, &data, &None);
    assert!(result.is_err());
  }

  #[test]
  fn test_len() {
    let columns = get_fake_columns();
    let data = get_fake_rows();
    let csv_file = CSVFile::build(&columns, &data, &None).unwrap();
    assert_eq!(csv_file.len(), 3);
  }

  #[test]
  fn test_count_rows() {
    let columns = get_fake_columns();
    let data = get_fake_rows();
    let csv_file = CSVFile::build(&columns, &data, &None).unwrap();
    assert_eq!(csv_file.count_rows(), 3);
  }

  #[test]
  fn test_has_column() {
    let columns = get_fake_columns();
    let data = get_fake_rows();
    let csv_file = CSVFile::build(&columns, &data, &None).unwrap();
    assert!(csv_file.has_column(&"a".to_string()));
    assert!(csv_file.has_column(&"b".to_string()));
    assert!(csv_file.has_column(&"c".to_string()));
    assert!(!csv_file.has_column(&"d".to_string()));
  }

  #[test]
  fn test_fill_column() {
    let columns = get_fake_columns();
    let data = get_fake_rows();
    let mut csv_file = CSVFile::build(&columns, &data, &None).unwrap();
    let new_data = vec!["10".to_string(), "11".to_string(), "12".to_string()];
    assert_eq!(csv_file.data[0][1], "2");
    assert_eq!(csv_file.data[1][1], "5");
    assert_eq!(csv_file.data[2][1], "8");
    csv_file.fill_column(&"b".to_string(), &new_data).unwrap();
    assert_eq!(csv_file.data[0][1], "10");
    assert_eq!(csv_file.data[1][1], "11");
    assert_eq!(csv_file.data[2][1], "12");
  }
}