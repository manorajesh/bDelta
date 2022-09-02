use std::{fs::File, io::Read, io::Seek, io::SeekFrom, io::Write};

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

    if new_len > source_len {
        while i < new_len.into() {
            new.read(&mut buffer2).expect("Unable to read file");
            diff.push((i, buffer2[0]));
            i += 1;
        }
    }
    diff
}

fn apply(diff: Vec<(u64, u8)>, file1: &str) {
    let mut file = File::create(file1).expect("Unable to read file");
    let mut buffer = [0; 1];

    for (i, c) in diff {
        file.seek(SeekFrom::Start(i as u64)).expect("Unable to seek file");
        file.read(&mut buffer).expect("Unable to read file");
        if buffer[0] != c {
            file.write_all(&[c as u8]).expect("Unable to write to file");
        }
    }
}