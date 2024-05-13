# A Renamer
Rename multiple files with values from an excel file.

![demo](demo.svg)

## Install
### Pre-built executables
You can download a pre-built executable avaliable from the [release page](release).

### Building from source
To install from source, you need Rust installed in your system. You can install Rust with [rustup](https://rustup.rs).\
The minimum rust version supported is `1.74.1`.
```text
$ git clone https://github.com/hufuhufu/arenamer
$ cd arenamer
$ cargo install --path .
```

## Usage
```text
Usage: arenamer.exe [OPTIONS] <WORKBOOK>

Arguments:
  <WORKBOOK>  Path to the workbook file

Options:
  -f, --files <FILES>      A regex to match with files within the target directory [default: .*]
  -d, --dir <DIR>          Target directory. Path of the target files to rename [default: .]
  -p, --pattern <PATTERN>  Output rename pattern. Use `?` as placeholder for the cell values, ex: `Invitation_?.pdf`. Every `?` will be replaced with the value from excel [default: ?]
  -s, --sheet <SHEET>      Name of the sheet to use [default: Sheet1]
  -c, --column <COLUMN>    Name of the column to use. Defaults to take the first column (assumes no column title/header)
  -h, --help               Print help
  -V, --version            Print version
```

As an example, using the test files available inside `tests` directory, this command below will rename all `.txt` files with `test_?.txt`, where `?` will be replaced with values under the column `name` inside `test.xlsx`.

```text
$ cd ./tests
$ arenamer -f "\.txt" -p "test_?.txt" -c "name" ./test.xlsx
```

And this command will revert it back
```text
$ arenamer -f "\.txt" -p "test_?.txt" -c "no" ./test.xlsx
```

## License
Copyright 2024. This project is released under MIT license.
