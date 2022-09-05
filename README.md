# bdel <p><sub><ins>B</ins>inary <ins>Del</ins>ta</sub><p>
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

### Generate
This subcommand generates the `diff.zip` file (filename and extension can be changed with `-o`) that can be sent to the recipient to be applied. Instead of a file output, stout can be chosen with the `-p` flag. For verification purposes, a [BLAKE3](https://en.wikipedia.org/wiki/BLAKE_(hash_function)#BLAKE3) hash of the target (or new) file is added to the zip.

### Apply
This subcommand takes in a `diff` file (either zip or plaintext) and applies it to the source file. The `diff` file is automatically removed; however, it can be left alone with the `-d` flag. Additionally, the `-r` flag asks the user for a confirmation before applying the updates. The function first writes to a `.buffer` file before updating the source. However, if the hash of the `.buffer` file does not match the hash in the `diff` file, the `.buffer` file is deleted, and the program exits without making any changes.

## Note
**Be careful when using this utility;** only basic testing was preformed.