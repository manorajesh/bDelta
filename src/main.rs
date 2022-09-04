use std::{fs::{OpenOptions, File}, io::{self, Read, Write}};

/* 
 First, identify and record the differing bytes between the source and
 the new file
 Then, create a new temporary file and copy the source file with the 
 differing bytes at the correct locations as well.
 Finally, rename the temporary file to the new file.
 */

fn main() {
    let source = "src/source.bin";
    let new = "src/new.bin";
    
    let diff = diff(source, new);
    println!("{:?}", diff);

    apply(diff, source, true, new);
}

fn diff(file1: &str, file2: &str) -> Vec<(u64, u8, bool)> {
    let mut source = File::open(file1).expect("Unable to read file");
    let mut new = File::open(file2).expect("Unable to read file");

    let mut buffer1 = [0; 1];
    let mut buffer2 = [0; 1];

    let source_len = source.metadata().unwrap().len();
    let new_len = new.metadata().unwrap().len();
    let mut diff = Vec::new();
    
    let mut i: u64 = 0;
    loop {
        if new.read(&mut buffer2).expect("Unable to read file") == 0 {break} // break when EOF
        if source.read(&mut buffer1).expect("Unable to read file") == 0 {break}

        while buffer1 != buffer2 {
            diff.push((i, buffer2[0], false));
            if new.read(&mut buffer2).expect("Unable to read file") == 0 {break} // break when EOF
            i += 1
        }
        i += 1;
    }

    if (new_len - diff.len() as u64) > source_len {
        while i < new_len.into() {
            diff.push((i, buffer2[0], false));
            new.read(&mut buffer2).expect("Unable to read file"); // already read the byte in loop
            i += 1;
        }
    } else if new_len < source_len {
        diff.push((new_len, 0, true));
    }

    if diff != Vec::new() { // If there are no differences, return an empty vector
        diff.insert(0, (i, 0, false)); // Add length of new file to the beginning of the diff
    }
    diff
}

fn apply(diff_bytes: Vec<(u64, u8, bool)>, target: &str, verify: bool, new: &str) {
    if diff_bytes == Vec::new() {
        println!("No differences found");
        return;
    }
    if !verify {
        drop(new); // Drop the new file if automatic verification is disabled
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
    }

    if verify {
        if diff(&buffer_file[..], new) == Vec::new() {
            std::fs::remove_file(target).expect("Unable to remove file");
            std::fs::rename(buffer_file, target).expect("Unable to rename file");
            println!("Verification successful");
            println!("Successfully applied patch at {}", target);
        } else {
            println!("Verification failed, removing buffer file");
            //std::fs::remove_file(&buffer_file).expect("Unable to remove file");
        }
    } else {
        let mut usr_input = String::new();

        loop {
            println!("Verification is disabled. Do you want to apply buffer? (y/n)");

            io::stdin()
                .read_line(&mut usr_input)
                .expect("Unable to read input");

            match usr_input.trim() {
                "y" => {
                    std::fs::remove_file(target).expect("Unable to remove file");
                    std::fs::rename(buffer_file, target).expect("Unable to rename file");
                    println!("{} file updated", target);
                    break;
                },
                "n" => {
                    std::fs::remove_file(&buffer_file).expect("Unable to remove file");
                    println!("{} file removed", buffer_file);
                    break;
                },
                _ => {
                    println!("Invalid input");
                    usr_input.clear();
                }
            }
        }

    }
}