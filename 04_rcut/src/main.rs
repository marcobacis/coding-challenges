use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    process::exit,
};

use clap::Parser;
use rcut::{field::parse_ranges, Cut};

#[derive(Parser, Debug)]
#[command(author, version, disable_help_flag = true)]
struct Args {
    /// The list specifies fields, separated by the field delimiter character (see -d).
    #[clap(short)]
    fields: Option<String>,

    #[clap(short)]
    bytes: Option<String>,

    /// Suppress lines with no field delimiter characters
    #[clap(short, action, requires("fields"))]
    suppress: bool,

    /// Use whitespace (spaces and tabs) as the delimiter
    #[clap(short, action, requires("fields"))]
    whitespace: bool,

    /// Use delim as the field delimiter character instead of the tab character.
    #[clap(short, requires("fields"))]
    delim: Option<String>,

    #[clap(short, action)]
    help: bool,

    files: Vec<String>,
}

fn usage() {
    println!("usage: rcut -b list [file ...]");
    println!("       rcut -c list [file ...]");
    println!("       rcut -f list [-s] [-w | -d delim] [file ...]");
}

fn main() {
    let args = Args::parse();
    check_args(&args);

    let command = Cut {
        delimiter: match args.delim {
            Some(s) => s.chars().into_iter().nth(0).unwrap_or('\t'),
            None => '\t',
        },
        ranges: parse_ranges(&args.fields.unwrap()),
        whitespace: args.whitespace,
        suppress: args.suppress,
        ..Cut::default()
    };

    let paths = match args.files.len() {
        0 => vec![String::from("-")],
        _ => args.files,
    };

    for path in paths {
        let reader: Box<dyn BufRead> = match path.as_str() {
            "-" => Box::new(BufReader::new(io::stdin())),
            _ => Box::new(BufReader::new(File::open(path).unwrap())),
        };
        for line in reader.lines() {
            if let Some(result) = command.execute_line(&line.unwrap()) {
                println!("{}", result);
            }
        }
    }
}

fn check_args(args: &Args) {
    // Only one of the three commadns can be called at one time
    let mut commands_called = 0;
    if args.fields.is_some() {
        commands_called += 1;
    }

    if args.help {
        usage();
        exit(0);
    }

    if commands_called != 1 {
        usage();
        exit(1);
    }
}
