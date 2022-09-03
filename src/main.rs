use std::{fs::{OpenOptions, File}, io::{Read, Seek, SeekFrom, Write}};

fn main() {
    let source = "src/source.bin";
    let new = "src/new.bin";
    
    let diff = diff(source, new);
    println!("{:?}", diff);

    apply(diff, source)
}

fn diff(file1: &str, file2: &str) -> Vec<(u64, u8)> {
    let mut source = File::open(file1).expect("Unable to read file");
    let mut new = File::open(file2).expect("Unable to read file");

    let mut buffer1 = [0; 1];
    let mut buffer2 = [0; 1];

    let source_len = source.metadata().unwrap().len();
    let new_len = new.metadata().unwrap().len();
    let mut diff = Vec::new();
    
    let mut i: u64 = 0;
    while i < source_len.into() {
        source.read(&mut buffer1).expect("Unable to read file");
        new.read(&mut buffer2).expect("Unable to read file");

        while buffer1 != buffer2 {
            diff.push((i, buffer2[0]));
            new.read(&mut buffer2).expect("Unable to read file");
            i += 1
        }
        i += 1;
    }

    if (new_len - diff.len() as u64) > source_len {
        while i < new_len.into() {
            new.read(&mut buffer2).expect("Unable to read file");
            diff.push((i, buffer2[0]));
            i += 1;
        }
    }
    diff
}

fn apply(diff: Vec<(u64, u8)>, target: &str) {
    let buffer_file = String::from(target) + ".buffer";
    let mut diff = diff;

    // write to buffer file
    let mut bfile = OpenOptions::new()
                            .read(true)
                            .write(true)
                            .create(true)
                            .open(buffer_file)
                            .expect("Unable to open file");

    // target file to read from
    let mut tfile = OpenOptions::new()
                            .read(true)
                            .open(target)
                            .expect("Unable to open file");

    let mut buffer = [0; 1];
    let tfile_len = tfile.metadata().unwrap().len() + diff.len() as u64;

    let mut i: u64 = 0;
    while i < tfile_len {
        if diff[0].0 == i {
            bfile.write(&[diff[0].1]).expect("Unable to write to file");
            diff.remove(0);
        } else {
            tfile.read(&mut buffer).expect("Unable to read file");
            bfile.write(&buffer).expect("Unable to write to file");
        }
        i += 1;
    }
}