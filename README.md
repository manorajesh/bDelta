# bdel <p><sub>Binary Delta</sub><p>
A Rust application that generates a [binary delta](https://en.wikipedia.org/wiki/Delta_encoding) and can apply it can create the target.

## Installation
`cargo install bdel`
<br>
or
<br>
Visit the [repository page](https://github.com/manorajesh/bDelta) and download the release for your platform.

## Usage
```
USAGE:
    bdel <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    apply       Apply a binary patch to a file
    generate    Generate a binary patch from a source and new file
    help        Print this message or the help of the given subcommand(s)
```

#### Generate
This subcommand generates the `diff.zip` file (filename can be changed with `-o`) that can be sent to the recipient to be applied. Instead of a file output, stout can be chosen with the `-p` flag.

#### Apply
This subcommand takes in a `diff` file (either zip or plaintext) and applies it to the source file. The `diff` file is automatically removed; however, it can be left alone with the `-d` flag. Additionally, the `-r` flag asks the user for a confirmation before applying the updates.

## Note
**Be careful when using this utility;** only basic testing was preformed.