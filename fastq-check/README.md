
## Description

Fastq quantity & format check.

## Examples:

1. build
```shell
# clone this repo and 
cargo build
```
2. run
```shell
fastq_check -1 R1.fastq -2 R2.fastq.gz
```

## Usage:

```shell
USAGE:
    fastq_check [OPTIONS] --read1 <FILE>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -1, --read1 <FILE>      first read of a pair
    -2, --read2 <FILE>      second read of a pair
    -n, --head <NUMBER>     only check first n reads
    -b, --base <NUMBER>     min base number threshold, default 0
    -r, --reads <NUMBER>    min reads number threshold, default 0
```

## Output:
```json
{
  "base_number": 6000000,
  "read1_base_number": 3000000,
  "read2_base_number": 3000000,
  "pair_readed": 20000,
  "c_reads": true,
  "c_base": true,
  "c_format": true,
  "summary": true
}
```
