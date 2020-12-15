# table-filter: Regex-based tsv/csv file filter

Table-filter(tf) is a regex-based tsv/csv filter that support first-row header indexing and 0-N indexing

## Usage

```shell
tf 0.1.1
slyo <sean.lyo@outlook.com>
Regex-based tsv/csv file filter

USAGE:
    tf [FLAGS] [OPTIONS] --exclude <exclude>... [--] [src]

FLAGS:
        --comma        Input is comma separated
    -h, --help         Prints help information
        --less         Give less result
        --no-header    Use 0-N indexed header instead of first row
        --tab          Input is tab separated (default)
    -V, --version      Prints version information

OPTIONS:
    -e, --exclude <exclude>...    Pattern for value excluding, overrided by `include` if overlapped
    -i, --include <include>...    Pattern for value including, toml format is required, e.g. 'header=(?:regex:pattern)'
    -o, --output <output>         output file, `-` mean stdout [default: -]

ARGS:
    <src>    Input file, `-` mean stdin [default: -]
```
