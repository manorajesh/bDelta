use blake3::Hasher;
use std::{
    fs::File,
    io::{Read, Write},
};
use zip::ZipWriter;

use crate::CHUNK_SIZE;

// Generate a diff from a source and new file

fn lcs(pattern1: [u8; CHUNK_SIZE as usize], pattern2: [u8; CHUNK_SIZE as usize], len1: usize, len2: usize) -> Vec<(u64, u8, bool)> {
    if len1 == 0 || len2 == 0 {
        return Vec::new();
    } else if pattern1[len1-1] == pattern2[len2-1] {
        let mut lcs = lcs(pattern1, pattern2, len1-1, len2-1);
        lcs.push((len1 as u64, pattern1[len1-1], false));
        return lcs;
    } else {
        let lcs1 = lcs(pattern1, pattern2, len1-1, len2);
        let lcs2 = lcs(pattern1, pattern2, len1, len2-1);
        if lcs1.len() > lcs2.len() {
            return lcs1;
        } else {
            return lcs2;
        }
    }
}
    
pub fn diff(file1: &str, file2: &str) -> Vec<(u64, u8, bool)> {
    let mut source = File::open(file1).expect("Unable to read file");
    let mut new = File::open(file2).expect("Unable to read file");

    let mut buffer1 = [0; CHUNK_SIZE as usize];
    let mut buffer2 = [0; CHUNK_SIZE as usize];

    let source_len = source.metadata().unwrap().len();
    let new_len = new.metadata().unwrap().len();

    let max_character = if source_len < CHUNK_SIZE {
        source_len
    } else {
        CHUNK_SIZE
    };

    let mut diff = Vec::new();

    println!("Finding diffs...");

    let mut i: u64 = 0;
    let mut j: usize = 0;
    loop {
        if new.read(&mut buffer2).expect("Unable to read file") == 0 {
            break;
        } // break when EOF
        if source.read(&mut buffer1).expect("Unable to read file") == 0 {
            break;
        }

        if buffer1 != buffer2 {
            diff.append(&mut lcs(buffer1, buffer2, buffer1.len(), buffer2.len()));
        } else {
            i += CHUNK_SIZE;
        }
    }

    if new_len > source_len {
        let mut g = j;
        while j < new_len as usize {
            while g < CHUNK_SIZE as usize && j < new_len as usize {
                diff.push((j as u64, buffer2[g], false));
//                println!("{} at {}", buffer2[g] as char, j);
                g += 1;
                j += 1;
            }
            new.read(&mut buffer2).expect("Unable to read file");
            g = 0;
        }
    } else if new_len < source_len {
        diff.push((new_len, 0, true));
    }

    if diff != Vec::new() {
        // If there are no differences, return an empty vector
        diff.insert(0, (new_len, 0, false)); // Add the length of the new file to the beginning of the vector
    }
    diff
}

pub fn serialize(diff: Vec<(u64, u8, bool)>, output_name: String, print_stdout: bool, new_file: &str) {
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

    println!("\nZipping patch...");

    zip.start_file("diff", Default::default())
        .expect("Unable to write to file");
    for (i, byte, flag) in diff {
        write!(zip, "{:x},{:x},{}\n", i, byte, flag as u8).expect("Unable to write to file");
    }

    println!("\nGenerating hash...");

    // Generate blake3 hash for new file and write to zip
    let mut hasher = Hasher::new();
    let mut new_file = File::open(new_file).expect("Unable to open file");
    let mut buffer = [0; CHUNK_SIZE as usize];
    loop {
        if new_file.read(&mut buffer).expect("Unable to read file") == 0 {
            break;
        } // break when EOF
        hasher.update(&buffer);
    }

    let hash = hasher.finalize();

    zip.start_file("hash", Default::default())
        .expect("Unable to write to file");

    println!("Hash: {}", hash.to_hex());
    write!(zip, "{}", hash.to_hex()).expect("Unable to write to file");
}
