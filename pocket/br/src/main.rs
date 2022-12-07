use std::env;
use std::fs;
use std::io;

fn main() -> io::Result<()> {
    let input_file_name = env::args().nth(1).expect("Missing input file name arg");
    let output_file_name = env::args().nth(2).expect("Missing output file name arg");

    let mut bytes = fs::read(input_file_name)?;
    for b in &mut bytes {
        let mut rev = 0;
        for _ in 0..8 {
            rev <<= 1;
            rev |= *b & 1;
            *b >>= 1;
        }
        *b = rev;
    }
    fs::write(output_file_name, bytes)
}
