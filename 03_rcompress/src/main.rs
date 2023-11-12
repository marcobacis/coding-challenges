use std::env;

use rcompress::{compress, decompress};

fn print_usage(args: &Vec<String>) {
    println!("Usage: {} compress|decompress input output", args[0]);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        print_usage(&args);
    }

    let method = args[1].clone();
    let path_input = args[2].clone();
    let path_output = args[3].clone();

    match method.as_str() {
        "compress" => compress(&path_input, &path_output).expect("Error during compression"),
        "decompress" => decompress(&path_input, &path_output).expect("Error during decompression"),
        _ => print_usage(&args),
    };
}
