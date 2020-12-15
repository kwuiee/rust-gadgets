# spdis

Extract splitted and discordant reads from sam/bam according to [lumpy](https://github.com/arq5x/lumpy-sv).

## Getting Started

Help message.

```shell
$ spdis --help
spdis 0.0.2
slyo <sean.lyo@outlook.com>
Extract splitted and discordant reads from sam/bam according to lumpy.

USAGE:
    spdis [FLAGS] [OPTIONS] <input> --discordant <file> --splitted <file>

FLAGS:
    -i, --include_dup       include duplicates
    -m, --min_nonoverlap    minimum non-overlap between split alignments on the query (default=20)
    -h, --help              Prints help information
    -V, --version           Prints version information

OPTIONS:
    -s, --splitted <file>          ouput splitted bam path
    -n, --split_number <number>    max split number of a split read
    -d, --discordant <file>        output discordant bam path

ARGS:
    <input>    input bam file

```
