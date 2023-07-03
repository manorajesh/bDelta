use clap::{Parser, Subcommand};
use std::{
    fs::remove_file,
    path::Path,
    process::exit,
};

mod apply;
mod diff;

use apply::{apply, deserialize};
use diff::{diff, serialize};
/*
First, identify and record the differing bytes between the source and
the new file
Then, create a new temporary file and copy the source file with the
differing bytes at the correct locations as well.
Finally, rename the temporary file to the new file.
*/

// TODO
// 1. DONE Add hashing to the diff to check if they are the same when applied
// 2. Issue with context not being applied correctly

#[derive(Parser)]
#[clap(version = "0.2.1", author = "Mano Rajesh")]
/// Generate or apply a binary patch to a file (binary delta)
struct Cli {
    #[clap(subcommand)]
    subcmd: SubCommands,
}

#[derive(Subcommand)]
enum SubCommands {
    /// Apply a binary patch to a file
    Apply {
        #[clap(value_name = "SOURCE")]
        source: String,

        #[clap(value_name = "DIFF")]
        diff: String,

        #[clap(short, long, action)]
        request: bool,

        #[clap(short, long = "dont-delete-diff", action, default_value = "false")]
        delete: bool,
    },
    /// Generate a binary patch from a source and new file
    Gen {
        #[clap(value_name = "SOURCE")]
        source: String,

        #[clap(value_name = "NEW")]
        new: String,

        #[clap(short, long, default_value = "diff.zip")]
        output: String,

        #[clap(short, long, action)]
        print_stdout: bool,
    },
}

pub const CHUNK_SIZE: u64 = 32;

fn main() {
    let cli = Cli::parse();

    match cli.subcmd {
        SubCommands::Apply {
            source,
            diff,
            request,
            mut delete,
        } => {
            let source = if Path::new(&source).exists() {
                &source[..]
            } else {
                eprintln!("Source file does not exist");
                exit(1);
            };

            let diff_file = if Path::new(&diff).exists() {
                &diff[..]
            } else {
                eprintln!("Diff file does not exist");
                exit(1);
            };

            let diff = deserialize(diff_file);

            if !apply(diff.0, source, request, diff.1) {
                delete = true;
            }

            if !delete {
                // for readability - if 'dont-delete' is false
                remove_file(diff_file).expect("Failed to delete diff file");
                println!("Deleted diff file");
            }
        }
        SubCommands::Gen {
            source,
            new,
            output,
            print_stdout,
        } => {
            let source = if Path::new(&source).exists() {
                &source[..]
            } else {
                eprintln!("Source file does not exist");
                exit(1);
            };

            let new = if Path::new(&new).exists() {
                &new[..]
            } else {
                eprintln!("New file does not exist");
                exit(1);
            };

            serialize(diff(source, new), output, print_stdout, new); // Add diff to zipped file
        }
    }
}