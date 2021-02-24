# Insize

Insert size stats and plot.

## Getting started

Help message

```shell
insize
slyo <sean.lyo@outlook.com>
Insert size(template length) consensus with little memory.

USAGE:
    insize [OPTIONS] <bam> -o <FILE>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -o <FILE>          Output svg file path.
    -m <NUMBER>        Maximum insert size to record. Bigger number costs more memory.

ARGS:
    <bam>    Input bam file.
```

Run test case with PNG output.

```
insize -o insert-size.png test.bam
```

SVG is also supported.

```shell
insize -o insert-size.svg tests/test.bam
```

## Benchmark

~ 20s/Gb
