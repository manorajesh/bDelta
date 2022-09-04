use std::{fs::{remove_file, rename, OpenOptions, File}, io::{stdout, stdin, Read, Write}, path::Path, process::exit};
use clap::{Parser, Subcommand};
use zip::{ZipArchive, ZipWriter};

/* 
 First, identify and record the differing bytes between the source and
 the new file
 Then, create a new temporary file and copy the source file with the 
 differing bytes at the correct locations as well.
 Finally, rename the temporary file to the new file.
 */

 /*
  Preformace Hogs:
  - memcpy when applying large update
 */

#[derive(Parser)]
#[clap(version = "0.1", author = "Mano Rajesh")]
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

        #[clap(short, long)]
        request: bool,
        
        #[clap(short, long = "delete-diff", action, default_value = "true")]
        delete: bool,
    },
    /// Generate a binary patch from a source and new file
    Generate {
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

fn main() {
    let cli = Cli::parse();

    match cli.subcmd {
        SubCommands::Apply { source, diff , request, delete} => {
            let source = if Path::new(&source).exists() {&source[..]} else {
                eprintln!("Source file does not exist");
                exit(1);
            };

            let diff_file = if Path::new(&diff).exists() {&diff[..]} else {
                eprintln!("New file does not exist");
                exit(1);
            };

            let diff = deserialize(diff_file);

            apply(diff, source, request);

            if delete {
                remove_file(diff_file).expect("Failed to delete diff file");
                println!("Deleted diff file");
            }
        }
        SubCommands::Generate { source, new , output, print_stdout} => {
            let source = if Path::new(&source).exists() {&source[..]} else {
                eprintln!("Source file does not exist");
                exit(1);
            };

            let new = if Path::new(&new).exists() {&new[..]} else {
                eprintln!("Diff file does not exist");
                exit(1);
            };

            serialize(diff(source, new), output, print_stdout); // Add diff to zipped file
        }
    }
}

fn diff(file1: &str, file2: &str) -> Vec<(u64, u8, bool)> {
    let mut source = File::open(file1).expect("Unable to read file");
    let mut new = File::open(file2).expect("Unable to read file");

    const CHUNK_SIZE: u64 = 1024;

    let mut buffer1 = [0; CHUNK_SIZE as usize];
    let mut buffer2 = [0; CHUNK_SIZE as usize];

    let source_len = source.metadata().unwrap().len();
    let new_len = new.metadata().unwrap().len();
    let mut diff = Vec::new();

    println!("Finding diffs...");
    
    let mut i: u64 = 0;
    let mut j: usize = 0;
    loop {
        if new.read(&mut buffer2).expect("Unable to read file") == 0 {break} // break when EOF
        if source.read(&mut buffer1).expect("Unable to read file") == 0 {break}

        if buffer1 != buffer2 {
            while i < CHUNK_SIZE as u64 && j < CHUNK_SIZE as usize {
                if buffer1[i as usize] != buffer2[j] {
                    diff.push((j as u64, buffer2[j], false));
                    j += 1;
                } else {
                    i += 1;
                    j += 1;
                }
            }
        } else {
            i += CHUNK_SIZE;
        }
    }
    
    if new_len > source_len {
        while j < CHUNK_SIZE as usize {
            diff.push((i + j as u64, buffer2[j], false));
            i += 1;
            j += 1;
        }
    } else if new_len < source_len {
        diff.push((new_len, 0, true));
    }

    if diff != Vec::new() { // If there are no differences, return an empty vector
        diff.insert(0, (new_len, 0, false)); // Add the length of the new file to the beginning of the vector
    }
    diff
}

fn apply(diff_bytes: Vec<(u64, u8, bool)>, target: &str, request: bool) {
    if diff_bytes == Vec::new() {
        println!("No differences found");
        return;
    }

    let buffer_file = String::from(target) + ".buffer";
    let mut diff_bytes = diff_bytes;

    // write to buffer file
    let mut bfile = OpenOptions::new()
                            .read(true)
                            .write(true)
                            .create(true)
                            .open(&buffer_file)
                            .expect("Unable to open file");

    // target file to read from
    let mut tfile = OpenOptions::new()
                            .read(true)
                            .open(target)
                            .expect("Unable to open file");

    let mut buffer = [0; 1];
    let max_character = diff_bytes[0].0;
    diff_bytes.remove(0);

    println!("Applying patch...");
    
    diff_bytes.push((0, 0, true)); // Add a dummy value to the end of the vector to prevent out of bounds errors
    let mut i: u64 = 0;
    while i < max_character {
        if diff_bytes[0].2 && i == max_character {
            break
        } else if diff_bytes[0].0 == i {
            bfile.write(&[diff_bytes[0].1]).expect("Unable to write to file");
            diff_bytes.remove(0);
            i += 1;
        } else {
            tfile.read(&mut buffer).expect("Unable to read file");
            bfile.write(&buffer).expect("Unable to write to file");
            i += 1;
        }

        if i % 100 == 0 {
            print!("\r{:.1}%", (i as f64 / max_character as f64) * 100.0);
            stdout().flush().unwrap();
        }
    }
    println!("\r100.0%");

    if request {
        let mut usr_input = String::new();

        loop {
            println!("Do you want to apply buffer? (y/n)");

            stdin()
                .read_line(&mut usr_input)
                .expect("Unable to read input");

            match usr_input.trim() {
                "y" => {
                    remove_file(target).expect("Unable to remove file");
                    rename(buffer_file, target).expect("Unable to rename file");
                    println!("{} file updated", target);
                    break;
                },
                "n" => {
                    remove_file(&buffer_file).expect("Unable to remove file");
                    println!("{} file removed", buffer_file);
                    break;
                },
                _ => {
                    println!("Invalid input");
                    usr_input.clear();
                }
            }
        }
    } else {
        remove_file(target).expect("Unable to remove file");
        rename(buffer_file, target).expect("Unable to rename file");
        println!("Successfully applied patch at {}", target);
    }
}

fn serialize(diff: Vec<(u64, u8, bool)>, output_name: String, print_stdout: bool) {
    if diff == Vec::new() {
        println!("No differences found");
    } else {
        println!("Number of character differences: {}", diff.len() - 1); // -1 because placeholder start element is included
    }

    if print_stdout {
        for (i, byte, flag) in diff {
            println!("{:x},{:x},{}", i, byte, flag as u8);
        }
        return;
    }

    let output = File::create(output_name).expect("Unable to create file");
    
    let mut zip = ZipWriter::new(output);

    zip.start_file("diff", Default::default()).expect("Unable to write to file");
    for (i, byte, flag) in diff {
        write!(zip, "{:x},{:x},{}\n", i, byte, flag as u8).expect("Unable to write to file");
    }
}

fn deserialize(zipped: &str) -> Vec<(u64, u8, bool)> {
    // Open the file
    let output = File::open(zipped).expect("Unable to open file");

    // Attempt to unzip the file
    let contents = match ZipArchive::new(output) {
        Ok(mut archive) => {
            let mut output = archive.by_index(0).expect("Unable to read zip file");

            // Read the file
            let mut contents = String::new();
            output.read_to_string(&mut contents).expect("Unable to read zip file");
            contents
        },
        Err(_) => {
            // If the file is not a zip file, read it as a normal file
            let mut contents = String::new();
            let mut text = File::open(zipped).expect("Unable to open file");
            text.read_to_string(&mut contents).expect("Unable to read file");
            contents
        }
    };

    let mut diff = Vec::new();

    for line in contents.lines() {
        let mut split = line.split(',');
        let i = u64::from_str_radix(split.next().unwrap(), 16).unwrap();
        let byte = u8::from_str_radix(split.next().unwrap(), 16).unwrap();
        let flag = split.next().unwrap().parse::<u8>().unwrap() == 1;
        diff.push((i, byte, flag));
    }
    diff
}