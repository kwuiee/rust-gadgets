# fqmerge: Merge fastq pairs in a directory into one pair.

## How to use

Command line help shows below

```shell
USAGE:
    fqmerge <srcdir> --read1 <FILE> --read2 <FILE>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --read1 <FILE>    read1 output path of concated pair
        --read2 <FILE>    read2 output path of concated pair

ARGS:
    <srcdir>    fastq pair source directory to concat, fastq glob `*_R[12].fastq.gz` or `*_R[12].fq.gz`
```

To run it just

```shell
fqmerge \
    --read1 /output/path/to/read1.gz \
    --read2 /output/path/to/read2.gz \
    /src/directory/contains/paired_fastq/
```
