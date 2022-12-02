use std::env;
use std::process;
use std::fs::File; // For read_file_lines()
use std::io::{self, BufRead}; // For read_file_lines()

fn read_wc(filename: &String) {
    let mut words = 0;
    let mut characters = 0;
    let mut lines = 0;
    // use ? is better than unwrap()
    let file = File::open(filename).expect("open file error.");
    for line in io::BufReader::new(file).lines() {
        let line_str: Vec<char> = line.expect("read error").chars().collect();

        // do something with line_str
        lines += 1;
        for i in 0..line_str.len() {
            if line_str[i] == ' ' {
                words += 1;
            } else if line_str[i].is_ascii_alphabetic() {
                characters += 1;
            }
        }

    }
    // for the last word in the lines
    words += lines; 
    println!("Find {} words, {} lines, {} characters in file {}", words, lines, characters, filename);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Too few arguments.");
        process::exit(1);
    }
    let filename = &args[1];
    // Your code here :)
    read_wc(filename);
}
