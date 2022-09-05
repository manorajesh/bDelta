use blake3::Hasher;
use std::{
    fs::{remove_file, rename, File, OpenOptions},
    io::{stdin, stdout, Read, Write},
    process::exit,
};
use zip::ZipArchive;

use crate::CHUNK_SIZE;

// Apply a binary patch to a file from a diff

pub fn apply(diff_bytes: Vec<(u64, u8, bool)>, target: &str, request: bool, hash: String) -> bool {
    if diff_bytes == Vec::new() {
        println!("No differences found");
        return true;
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

    let mut buffer = [0; CHUNK_SIZE as usize];
    let max_character = diff_bytes[0].0;
    diff_bytes.remove(0);

    println!("Applying patch...");

    let mut i: u64 = 0;

    diff_bytes.push((0, 0, true)); // Add a dummy value to the end of the vector to prevent out of bounds error
    'bufupdate: loop {
        if tfile.read(&mut buffer).expect("Unable to read file") == 0 && i == max_character {
            break;
        } // break when EOF

        let mut g = 0;
        while i < max_character && g < CHUNK_SIZE {
            if diff_bytes[0].2 {
                break 'bufupdate;
            } else if diff_bytes[0].0 == i {
                bfile
                    .write(&[diff_bytes[0].1])
                    .expect("Unable to write to file");
                diff_bytes.remove(0);
                i += 1;
                g += 1;
            } else {
                bfile
                    .write(&[buffer[i as usize]])
                    .expect("Unable to write to file");
                i += 1;
                g += 1;
            }
            if i % 100 == 0 {
                print!("\r{:.1}%", (i as f64 / max_character as f64) * 100.0);
                stdout().flush().unwrap();
            }
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
                    break;
                }
                "n" => {
                    remove_file(&buffer_file).expect("Unable to remove file");
                    println!("{} file removed", buffer_file);
                    break;
                }
                _ => {
                    println!("Invalid input");
                    usr_input.clear();
                }
            }
        }
    } else {
        println!("\nVerifying hash...");
        if hash != String::new() {
            let new_hash = hash_file(&buffer_file);

            if compare_hash(&hash, &new_hash) {
                println!("Verification successful\n");
            } else {
                println!("\nVerification failed; removing buffer file");
                remove_file(&buffer_file).expect("Unable to remove file");
                println!("{} file removed", buffer_file);

                println!("\nDouble check that the diff file is correct");
                println!("Hash found in diff: {}", hash);
                println!("Hash of buffer file: {}", new_hash);
                return false; // don't delete the zip file for debugging purposes
            }
        } else {
            println!("No hash provided; Skipping...")
        }

        println!("Applying buffer...");
        remove_file(target).expect("Unable to remove file");
        rename(buffer_file, target).expect("Unable to rename file");
        println!("Successfully applied patch at {}", target);
    }
    true // to delete the zip file
}

fn hash_file(file: &str) -> String {
    let mut hasher = Hasher::new();
    let mut file = File::open(file).expect("Unable to open file");
    let mut buffer = [0; CHUNK_SIZE as usize];

    loop {
        if file.read(&mut buffer).expect("Unable to read file") == 0 {
            break;
        } // break when EOF
        hasher.update(&buffer);
    }

    let hash2 = format!("{}", hasher.finalize().to_hex());
    hash2
}

fn compare_hash(hash1: &String, hash2: &String) -> bool {
    if hash1 == hash2 {
        true
    } else {
        false
    }
}

pub fn deserialize(zipped: &str) -> (Vec<(u64, u8, bool)>, String) {
    // Open the file
    let output = File::open(zipped).expect("Unable to open file");

    // Attempt to unzip the file
    let contents = match ZipArchive::new(output) {
        Ok(mut archive) => {
            let mut output = archive.by_index(0).expect("Unable to read zip file");

            // Read the file
            let mut contents = String::new();
            output
                .read_to_string(&mut contents)
                .expect("Unable to read zip file");
            contents
        }
        Err(_) => {
            // If the file is not a zip file, read it as a normal file
            let mut contents = String::new();
            let mut text = File::open(zipped).expect("Unable to open file");
            text.read_to_string(&mut contents)
                .expect("Unable to read file");
            contents
        }
    };

    let hash = match ZipArchive::new(File::open(zipped).expect("Unable to open file")) {
        Ok(mut archive) => {
            let mut output = archive.by_index(1).expect("Unable to read zip file");

            // Read the file
            let mut contents = String::new();
            output
                .read_to_string(&mut contents)
                .expect("Unable to read zip file");
            contents
        }
        Err(_) => String::new(),
    };

    if hash == String::new() {
        loop {
            println!("No hash found in diff file; do you want to continue? (y/n)");

            let mut usr_input = String::new();

            stdin()
                .read_line(&mut usr_input)
                .expect("Unable to read input");

            match usr_input.trim() {
                "y" => {
                    break;
                }
                "n" => {
                    println!("Exiting...");
                    exit(0);
                }
                _ => {
                    println!("Invalid input");
                }
            }
        }
    }

    let mut diff = Vec::new();

    for line in contents.lines() {
        let mut split = line.split(',');
        let i = u64::from_str_radix(split.next().unwrap(), 16).expect("Check if diff file is correct");
        let byte = u8::from_str_radix(split.next().unwrap(), 16).expect("Check if diff file is correct");
        let flag = split.next().unwrap().parse::<u8>().unwrap() == 1;
        diff.push((i, byte, flag));
    }
    (diff, hash)
}
