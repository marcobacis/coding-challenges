# Coding Challenge #3 - Build a compression tool 

Solution for Coding Challenge #3 by Jhon Crickett ([here](https://codingchallenges.substack.com/p/coding-challenge-3) the description).

I created a tool called *rcompress* (rust + compress) to encode and decode files using the huffman coding technique.

## Building
The project is implemented in rust, so to build it use cargo
```
cargo build --release
```

## Usage

```
Usage: rcompress --input <INPUT> --output <OUTPUT> [compress|decompress]

Commands:
  compress    
  decompress  
  help        Print this message or the help of the given subcommand(s)

Options:
  -i, --input <INPUT>    Path of the input file
  -o, --output <OUTPUT>  Path of the output file
  -h, --help             Print help
  -V, --version          Print version
```
