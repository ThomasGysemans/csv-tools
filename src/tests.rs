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
        assert_eq!(csv_file.rows, data);
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
        csv_file.rows.clear();
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
        assert_eq!(csv_file.rows[0][1], "2");
        assert_eq!(csv_file.rows[1][1], "5");
        assert_eq!(csv_file.rows[2][1], "8");
        csv_file.fill_column(&"b".to_string(), &new_data).unwrap();
        assert_eq!(csv_file.rows[0][1], "10");
        assert_eq!(csv_file.rows[1][1], "11");
        assert_eq!(csv_file.rows[2][1], "12");
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
        csv_file.rows[0].remove(1);
        assert!(!csv_file.check_validity());
    }

    #[test]
    fn test_add_row() {
        let columns = get_fake_columns();
        let data = get_fake_rows();
        let mut csv_file = CSVFile::build(&columns, &data, &',').unwrap();
        let new_row = vec!["10".to_string(), "11".to_string(), "12".to_string()];
        csv_file.add_row(&new_row).unwrap();
        assert_eq!(csv_file.rows[3], new_row);
    }

    #[test]
    fn test_add_row_with_invalid_data() {
        let columns = get_fake_columns();
        let data = get_fake_rows();
        let mut csv_file = CSVFile::build(&columns, &data, &',').unwrap();
        let new_row = vec![
            "10".to_string(),
            "11".to_string(),
            "12".to_string(),
            "13".to_string(),
        ];
        assert!(csv_file.add_row(&new_row).is_err());
    }

    #[test]
    fn test_add_column() {
        let columns = get_fake_columns();
        let data = get_fake_rows();
        let mut csv_file = CSVFile::build(&columns, &data, &',').unwrap();
        csv_file.add_column(&"d".to_string()).unwrap();
        assert_eq!(csv_file.columns[3], "d");
        assert_eq!(csv_file.rows[0][3].len(), 0);
        assert_eq!(csv_file.rows[1][3].len(), 0);
        assert_eq!(csv_file.rows[2][3].len(), 0);
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
        assert_eq!(csv_file.rows.len(), 3); // there is still 3 rows, but each row got extended by 1
        assert_eq!(csv_file.rows[0].len(), 4);
        assert_eq!(csv_file.rows[1].len(), 4);
        assert_eq!(csv_file.rows[2].len(), 4);
        assert_eq!(csv_file.rows[0][0].len(), 0); // empty strings
        assert_eq!(csv_file.rows[1][0].len(), 0);
        assert_eq!(csv_file.rows[2][0].len(), 0);
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
        assert_eq!(csv_file.rows.len(), 3);
        assert_eq!(csv_file.rows[0].len(), 2);
        assert_eq!(csv_file.rows[1].len(), 2);
        assert_eq!(csv_file.rows[2].len(), 2);
    }

    #[test]
    fn test_remove_row() {
        let columns = get_fake_columns();
        let data = get_fake_rows();
        let mut csv_file = CSVFile::build(&columns, &data, &',').unwrap();
        assert_eq!(csv_file.rows.len(), 3);
        csv_file.remove_row(0).unwrap();
        assert_eq!(csv_file.rows.len(), 2);
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
        assert_eq!(
            csv_file.get_cell(&CSVCoords { row: 0, column: 0 }).unwrap(),
            "1"
        );
        assert_eq!(
            csv_file.get_cell(&CSVCoords { row: 1, column: 1 }).unwrap(),
            "5"
        );
        assert_eq!(
            csv_file.get_cell(&CSVCoords { row: 2, column: 2 }).unwrap(),
            "9"
        );
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

    #[test]
    fn test_trim_start() {
        let columns = get_fake_columns();
        let data = vec![
            vec![empty_string(), empty_string(), empty_string()],
            vec![empty_string(), "8".to_string(), empty_string()],
            vec![empty_string(), empty_string(), empty_string()],
            vec!["1".to_string(), "2".to_string(), "3".to_string()],
            vec!["4".to_string(), "5".to_string(), "6".to_string()],
            vec!["7".to_string(), "8".to_string(), "9".to_string()],
        ];
        let mut csv_file = CSVFile::build(&columns, &data, &',').unwrap();
        assert_eq!(csv_file.count_rows(), 6);
        csv_file.trim_start();
        assert_eq!(csv_file.count_rows(), 5);
    }

    #[test]
    fn test_trim_start_with_a_file_of_one_nonempty_line() {
        let columns = get_fake_columns();
        let data = vec![vec!["1".to_string(), "2".to_string(), "3".to_string()]];
        let mut csv_file = CSVFile::build(&columns, &data, &',').unwrap();
        assert_eq!(csv_file.count_rows(), 1);
        csv_file.trim_start();
        assert_eq!(csv_file.count_rows(), 1);
    }

    #[test]
    fn test_trim_start_with_a_file_of_one_empty_line() {
        let columns = get_fake_columns();
        let data = vec![vec![empty_string(), empty_string(), empty_string()]];
        let mut csv_file = CSVFile::build(&columns, &data, &',').unwrap();
        assert_eq!(csv_file.count_rows(), 1);
        csv_file.trim_start();
        assert_eq!(csv_file.count_rows(), 0);
        assert!(csv_file.has_no_rows())
    }

    #[test]
    fn test_trim() {
        let columns = get_fake_columns();
        let data = vec![
            vec![empty_string(), empty_string(), empty_string()],
            vec![empty_string(), "8".to_string(), empty_string()],
            vec![empty_string(), empty_string(), empty_string()],
            vec!["1".to_string(), "2".to_string(), "3".to_string()],
            vec!["4".to_string(), "5".to_string(), "6".to_string()],
            vec!["7".to_string(), "8".to_string(), "9".to_string()],
            vec![empty_string(), empty_string(), empty_string()],
            vec![empty_string(), "8".to_string(), empty_string()],
            vec![empty_string(), empty_string(), empty_string()],
        ];
        let mut csv_file = CSVFile::build(&columns, &data, &',').unwrap();
        assert_eq!(csv_file.count_rows(), 9);
        csv_file.trim();
        assert_eq!(csv_file.count_rows(), 7);
        assert_eq!(
            csv_file.rows[0],
            vec![empty_string(), "8".to_string(), empty_string()]
        );
        assert_eq!(
            csv_file.rows[6],
            vec![empty_string(), "8".to_string(), empty_string()]
        );
    }

    #[test]
    fn remove_all_empty_lines() {
        let columns = get_fake_columns();
        let data = vec![
            vec![empty_string(), empty_string(), empty_string()],
            vec![empty_string(), "8".to_string(), empty_string()],
            vec![empty_string(), empty_string(), empty_string()],
            vec!["1".to_string(), "2".to_string(), "3".to_string()],
            vec!["4".to_string(), "5".to_string(), "6".to_string()],
            vec!["7".to_string(), "8".to_string(), "9".to_string()],
            vec![empty_string(), empty_string(), empty_string()],
            vec![empty_string(), "8".to_string(), empty_string()],
            vec![empty_string(), empty_string(), empty_string()],
        ];
        let mut csv_file = CSVFile::build(&columns, &data, &',').unwrap();
        assert_eq!(csv_file.count_rows(), 9);
        csv_file.remove_empty_lines();
        assert_eq!(csv_file.count_rows(), 5);
        assert_eq!(
            csv_file.rows[0],
            vec![empty_string(), "8".to_string(), empty_string()]
        );
        assert_eq!(
            csv_file.rows[1],
            vec!["1".to_string(), "2".to_string(), "3".to_string()]
        );
        assert_eq!(
            csv_file.rows[4],
            vec![empty_string(), "8".to_string(), empty_string()]
        );
    }

    #[test]
    fn test_merge_csv_files_with_same_number_of_rows() {
        let initial_columns = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let initial_rows = vec![
            vec!["1".to_string(), "2".to_string(), "3".to_string()],
            vec!["4".to_string(), "5".to_string(), "6".to_string()],
            vec!["7".to_string(), "8".to_string(), "9".to_string()],
        ];
        let mut csv_file1 = CSVFile::build(&initial_columns, &initial_rows, &',').unwrap();

        let other_columns = vec!["d".to_string(), "e".to_string()];
        let other_rows = vec![
            vec!["1".to_string(), "2".to_string()],
            vec!["4".to_string(), "5".to_string()],
            vec!["7".to_string(), "8".to_string()],
        ];
        let csv_file2 = CSVFile::build(&other_columns, &other_rows, &',').unwrap();

        csv_file1.merge(&csv_file2).unwrap();

        assert_eq!(csv_file1.len(), 5);
        assert_eq!(csv_file1.count_rows(), 3);
    }

    #[test]
    fn test_merge_csv_files_with_less_rows_than_other() {
        let initial_columns = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let initial_rows = vec![
            vec!["1".to_string(), "2".to_string(), "3".to_string()],
            vec!["4".to_string(), "5".to_string(), "6".to_string()],
        ];
        let mut csv_file1 = CSVFile::build(&initial_columns, &initial_rows, &',').unwrap();

        let other_columns = vec!["d".to_string(), "e".to_string()];
        let other_rows = vec![
            vec!["1".to_string(), "2".to_string()],
            vec!["4".to_string(), "5".to_string()],
            vec!["7".to_string(), "8".to_string()],
        ];
        let csv_file2 = CSVFile::build(&other_columns, &other_rows, &',').unwrap();

        csv_file1.merge(&csv_file2).unwrap();

        assert_eq!(csv_file1.len(), 5);
        assert_eq!(csv_file1.count_rows(), 3);
        assert_eq!(csv_file1.rows[2].len(), 5);
        assert!(csv_file1.check_validity());
    }

    #[test]
    fn test_merge_csv_files_with_more_rows_than_other() {
        let initial_columns = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let initial_rows = vec![
            vec!["1".to_string(), "2".to_string(), "3".to_string()],
            vec!["4".to_string(), "5".to_string(), "6".to_string()],
            vec!["7".to_string(), "8".to_string(), "9".to_string()],
        ];
        let mut csv_file1 = CSVFile::build(&initial_columns, &initial_rows, &',').unwrap();

        let other_columns = vec!["d".to_string(), "e".to_string()];
        let other_rows = vec![
            vec!["1".to_string(), "2".to_string()],
            vec!["4".to_string(), "5".to_string()],
        ];
        let csv_file2 = CSVFile::build(&other_columns, &other_rows, &',').unwrap();

        csv_file1.merge(&csv_file2).unwrap();

        assert_eq!(csv_file1.len(), 5);
        assert_eq!(csv_file1.count_rows(), 3);
        assert_eq!(csv_file1.rows[2].len(), 5);
        assert_eq!(csv_file1.rows[2][2], "9".to_string());
        assert_eq!(csv_file1.rows[2][3], empty_string());
        assert_eq!(csv_file1.rows[2][4], empty_string());
        assert!(csv_file1.check_validity());
    }

    #[test]
    fn test_to_map() {
        let columns = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let rows = vec![
            vec!["1".to_string(), "2".to_string(), "3".to_string()],
            vec!["4".to_string(), "5".to_string(), "6".to_string()],
            vec!["7".to_string(), "8".to_string(), "9".to_string()],
        ];

        let csv_file = CSVFile::build(&columns, &rows, &',').unwrap();
        let result = csv_file.to_map(|row: &String| row.parse::<u32>().unwrap());

        assert_eq!(result.len(), 3);
        assert_eq!(result.get(&String::from("a")).unwrap(), &vec![1, 4, 7]);
        assert_eq!(result.get(&String::from("b")).unwrap(), &vec![2, 5, 8]);
        assert_eq!(result.get(&String::from("c")).unwrap(), &vec![3, 6, 9]);
    }

    #[test]
    fn test_map_columns() {
        let columns = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let rows = vec![
            vec!["1".to_string(), "2".to_string(), "3".to_string()],
            vec!["4".to_string(), "5".to_string(), "6".to_string()],
            vec!["7".to_string(), "8".to_string(), "9".to_string()],
        ];

        let csv_file = CSVFile::build(&columns, &rows, &',').unwrap();
        let map: HashMap<String, Vec<String>> = csv_file.map_columns();

        assert_eq!(map.len(), 3);
        assert!(map.contains_key("a"));
        assert!(map.contains_key("b"));
        assert!(map.contains_key("c"));
        assert_eq!(map.get("a").unwrap().len(), 0);
        assert_eq!(map.get("b").unwrap().len(), 0);
        assert_eq!(map.get("c").unwrap().len(), 0);
    }

    #[test]
    fn test_to_map_string() {
        let columns = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let rows = vec![
            vec!["1".to_string(), "2".to_string(), "3".to_string()],
            vec!["4".to_string(), "5".to_string(), "6".to_string()],
            vec!["7".to_string(), "8".to_string(), "9".to_string()],
        ];

        let csv_file = CSVFile::build(&columns, &rows, &',').unwrap();
        let map = csv_file.to_map(|row: &String| row.clone());

        assert_eq!(map.len(), 3);
        assert!(map.contains_key("a"));
        assert!(map.contains_key("b"));
        assert!(map.contains_key("c"));
        assert_eq!(map.get("a").unwrap(), &vec!["1", "4", "7"]);
        assert_eq!(map.get("b").unwrap(), &vec!["2", "5", "8"]);
        assert_eq!(map.get("c").unwrap(), &vec!["3", "6", "9"]);
    }

    #[test]
    fn test_readme() {
        let filename = String::from("./test_langs.csv");
        let file = CSVFile::new(&filename, &',').unwrap();

        assert_eq!(
            file.columns,
            vec![
                "language".to_string(),
                "level_of_fun".to_string(),
                "level_of_difficulty".to_string()
            ]
        );

        assert_eq!(
            file.rows,
            vec![
                vec!["C++".to_string(), "10".to_string(), "8".to_string()],
                vec!["Rust".to_string(), "10".to_string(), "9".to_string()],
                vec!["JavaScript".to_string(), "9".to_string(), "1".to_string()],
                vec!["TypeScript".to_string(), "10".to_string(), "1".to_string()],
                vec!["Java".to_string(), "0".to_string(), "2".to_string()],
                vec!["HTML".to_string(), "10".to_string(), "-1".to_string()],
                vec!["GDScript".to_string(), "10".to_string(), "1".to_string()],
                vec!["Lua".to_string(), "7".to_string(), "1".to_string()],
            ]
        );

        assert_eq!(
            file.get_cell(&CSVCoords { row: 0, column: 0 }),
            Some(&"C++".to_string())
        );

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
    }
}
