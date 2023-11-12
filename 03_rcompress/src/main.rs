use clap::{Parser, Subcommand};
use rcompress::{compress, decompress};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // Command to execute
    #[command(subcommand)]
    command: Option<Commands>,

    /// Path of the input file
    #[arg(short, long)]
    input: String,

    /// Path of the output file
    #[arg(short, long)]
    output: String,
}
#[derive(Subcommand, Debug)]
enum Commands {
    Compress,
    Decompress,
}

fn main() {
    let args = Args::parse();

    let path_input = args.input;
    let path_output = args.output;

    match args.command {
        Some(Commands::Compress) | None => {
            compress(&path_input, &path_output).expect("Error during compression")
        }
        Some(Commands::Decompress) => {
            decompress(&path_input, &path_output).expect("Error during decompression")
        }
    };
}
