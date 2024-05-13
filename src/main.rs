use std::{
    collections::HashMap,
    fs::{rename, ReadDir},
};

use calamine::{open_workbook_auto, Data, RangeDeserializerBuilder, Reader};
use clap::Parser;
use regex::Regex;
use serde_derive::Deserialize;

#[derive(Debug, Parser)]
#[command(version, about)]
/// Rename multiple files with values from excel file.
struct Args {
    /// Path to the workbook file.
    workbook: String,

    /// A regex to match with files within the target directory.
    #[arg(default_value_t = String::from(".*"))]
    #[arg(short, long)]
    files: String,

    /// Target directory. Path of the target files to rename.
    #[arg(default_value_t = String::from("."))]
    #[arg(short, long)]
    dir: String,

    #[arg(default_value_t = String::from("?"))]
    #[arg(short, long)]
    /// Output rename pattern.
    /// Use ? as placeholder for the cell values, ex: `Invitation_?.pdf`.
    /// Every single ? will be replaced with the value from excel.
    pattern: String,

    /// Name of the sheet to use.
    #[arg(default_value_t = String::from("Sheet1"))]
    #[arg(short, long)]
    sheet: String,

    /// Name of the column to use.
    /// Defaults to take the first column (assumes no column title/header).
    #[arg(short, long)]
    column: Option<String>,
}

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error(transparent)]
    Calamine(#[from] calamine::Error),
    #[error(transparent)]
    CelemineDe(#[from] calamine::DeError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Regex(#[from] regex::Error),
    #[error("{0}")]
    Other(String),
}

#[derive(Debug, Deserialize)]
struct RangeValues {
    #[serde(flatten)]
    values: HashMap<String, Data>,
}

fn filter_filename(dir: ReadDir, regex: Regex) -> Vec<String> {
    let mut filenames: Vec<_> = dir
        .flatten()
        .filter(|entry| !entry.path().is_dir())
        .flat_map(|entry| entry.file_name().into_string())
        .filter(|filename| regex.is_match(filename))
        .collect();
    filenames.sort();
    filenames
}

fn get_values_from_excel(
    workbook_path: String,
    sheet: String,
    column: Option<String>,
) -> Result<Vec<String>, Error> {
    let mut workbook = open_workbook_auto(workbook_path)?;
    let range = workbook.worksheet_range(&sheet)?;

    if range.is_empty() {
        return Err(Error::Other("Workbook is emtpy".into()));
    }

    let mut res = vec![];
    if let Some(column) = column {
        let range = RangeDeserializerBuilder::new().from_range::<_, RangeValues>(&range)?;

        for val in range {
            res.push(
                val?.values
                    .get(&column)
                    .map(ToString::to_string)
                    .unwrap_or_default(),
            );
        }
    } else {
        let start = range.start().unwrap();
        let end = range.end().unwrap();
        let range = range.range(start, (end.0, start.1));

        for (_, _, cell) in range.cells() {
            res.push(cell.to_string());
        }
    }

    Ok(res)
}

fn map_value_to_pattern(values: Vec<String>, pattern: String) -> Vec<String> {
    values
        .into_iter()
        .map(|val| pattern.replace('?', &val))
        .collect()
}

fn main() -> Result<(), Error> {
    let args = Args::parse();

    let regex_input = Regex::new(&args.files)?;
    let dir = std::fs::read_dir(&args.dir)?;
    let filenames = filter_filename(dir, regex_input);
    let values = get_values_from_excel(args.workbook, args.sheet, args.column)?;

    let new_filenames = map_value_to_pattern(values, args.pattern);
    for (from, to) in filenames.into_iter().zip(new_filenames) {
        rename(&from, &to)?;
        println!("File renamed {from} -> {to}");
    }

    Ok(())
}

#[cfg(test)]
mod test {
    fn setup_workbook() -> (String, String, String) {
        let workbook_path = format!("{}/tests/test.xlsx", env!("CARGO_MANIFEST_DIR"));
        let sheet = "Sheet1".to_owned();
        let column = "name".to_owned();

        (workbook_path, sheet, column)
    }

    #[test]
    fn testing_filter_filename() {
        let regex = regex::Regex::new(".*").unwrap();
        let dir = std::fs::read_dir(".").unwrap();
        let filenames = crate::filter_filename(dir, regex);
        dbg!(&filenames);

        for filename in filenames.iter() {
            assert!(std::fs::metadata(filename).unwrap().is_file())
        }
    }

    #[test]
    fn testing_load_workbook() {
        let (workbook_path, sheet, column) = setup_workbook();
        let values = crate::get_values_from_excel(workbook_path, sheet, Some(column)).unwrap();
        dbg!(&values);

        assert_eq!(
            values,
            vec!["ABC", "DEF", "GHI", "JKL", "MNO", "", "PQR", "STU", "VWX", "YZ"]
        )
    }

    #[test]
    fn testing_pattern_and_value() {
        let (workbook_path, sheet, column) = setup_workbook();
        let values = crate::get_values_from_excel(workbook_path, sheet, Some(column)).unwrap();

        let pattern = "test ?.txt".into();
        let new_filename = crate::map_value_to_pattern(values, pattern);
        dbg!(&new_filename);

        assert_eq!(
            new_filename,
            vec![
                "test ABC.txt",
                "test DEF.txt",
                "test GHI.txt",
                "test JKL.txt",
                "test MNO.txt",
                "test .txt",
                "test PQR.txt",
                "test STU.txt",
                "test VWX.txt",
                "test YZ.txt",
            ]
        )
    }
}
