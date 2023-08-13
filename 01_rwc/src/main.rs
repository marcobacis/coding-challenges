use getopt::Opt;
use std::{
    env, fs,
    io::{self, BufRead, BufReader, Error},
    ops::Add,
    process::exit,
};

#[derive(Debug)]
struct Args {
    exe_name: String,
    count_bytes: bool,
    count_lines: bool,
    count_chars: bool,
    count_words: bool,
    paths: Vec<String>,
}

fn main() {
    let args = get_args();

    // If no path is given, count from stdin
    if args.paths.len() == 0 {
        let stdin_count = count_stdin();
        print_count(&stdin_count, &args);
        return;
    }

    // Count from all given files
    let counts: Vec<Result<CountResult, std::io::Error>> = args
        .paths
        .iter()
        .map(|path| count_file(path.to_string()))
        .collect();

    counts.iter().for_each(|count| print_count(&count, &args));

    // Show total count if multiple files
    if counts.len() > 1 {
        let sum = sum_counts(counts);
        print_count(&Ok(sum), &args);
    }
}

fn sum_counts(counts: Vec<Result<CountResult, Error>>) -> CountResult {
    let sum: CountResult = counts.iter().filter_map(|count| count.as_ref().ok()).fold(
        CountResult {
            path: "total".to_string(),
            bytes: 0,
            words: 0,
            chars: 0,
            lines: 0,
        },
        |acc, count| acc + count.clone(),
    );
    sum
}

fn get_args() -> Args {
    let mut args = Args {
        exe_name: "".to_string(),
        count_bytes: false,
        count_lines: false,
        count_chars: false,
        count_words: false,
        paths: Vec::new(),
    };

    let mut cmd_args: Vec<String> = env::args().collect();
    args.exe_name = cmd_args[0].clone();

    let mut opts = getopt::Parser::new(&cmd_args, "clmwh");
    loop {
        match opts.next().transpose().unwrap() {
            None => break,
            Some(opt) => match opt {
                Opt('c', None) => args.count_bytes = true,
                Opt('l', None) => args.count_lines = true,
                Opt('m', None) => args.count_chars = true,
                Opt('w', None) => args.count_words = true,
                Opt('h', None) => {
                    print_usage();
                    exit(0);
                }
                _ => unreachable!(),
            },
        }
    }
    args.paths = cmd_args.split_off(opts.index());
    args
}

fn print_usage() {
    println!(
        "Usage: {} [-c] [-l] [-m] [-w] [FILES]",
        env::args().collect::<Vec<String>>()[0]
    );
}

#[derive(Debug, Clone)]
struct CountResult {
    path: String,
    bytes: u64,
    words: u64,
    chars: u64,
    lines: u64,
}

impl CountResult {
    pub fn new(path: &str) -> CountResult {
        CountResult {
            path: path.to_string(),
            bytes: 0,
            words: 0,
            chars: 0,
            lines: 0,
        }
    }

    pub fn print(&self, options: &Args) -> String {
        let default_option = !options.count_bytes
            && !options.count_lines
            && !options.count_chars
            && !options.count_words;

        let mut fields: Vec<String> = Vec::new();

        if default_option || options.count_lines {
            fields.push(self.lines.to_string());
        }

        if default_option || options.count_words {
            fields.push(self.words.to_string());
        }

        if options.count_chars {
            fields.push(self.chars.to_string());
        } else if default_option || (options.count_bytes && !options.count_chars) {
            fields.push(self.bytes.to_string());
        }

        fields.push(self.path.clone());

        fields.join("\t")
    }
}

impl Add for CountResult {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            path: self.path,
            bytes: self.bytes + other.bytes,
            words: self.words + other.words,
            chars: self.chars + other.chars,
            lines: self.lines + other.lines,
        }
    }
}

fn count_file(path: String) -> Result<CountResult, std::io::Error> {
    // Try to open file
    match fs::metadata(&path) {
        Ok(metadata) => {
            if metadata.is_dir() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("{}: is a directory", path),
                ));
            }
        }
        Err(err) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("{}: open: no such file or directory", path),
            ))
        }
    }

    // Count file stats
    let file = fs::File::open(&path).unwrap();
    let mut reader = BufReader::new(file);
    Ok(count_buf(path, &mut reader))
}

fn count_stdin() -> Result<CountResult, std::io::Error> {
    let mut reader = Box::new(BufReader::new(io::stdin()));
    Ok(count_buf("".to_string(), &mut reader))
}

fn count_buf<R: BufRead>(name: String, reader: &mut R) -> CountResult {
    let mut res: CountResult = CountResult::new(name.as_str());

    let mut buf = Vec::<u8>::new();

    while reader
        .read_until(b'\n', &mut buf)
        .expect("read_until failed")
        != 0
    {
        let line = String::from_utf8(buf).expect("from_utf8 failed");

        res.lines += 1;
        res.chars += line.chars().count() as u64;
        res.bytes += line.bytes().count() as u64;
        res.words += line.split_whitespace().count() as u64;

        // return ownership to buf
        buf = line.into_bytes();
        buf.clear();
    }
    res
}

fn print_count(count: &Result<CountResult, std::io::Error>, options: &Args) {
    match count {
        Ok(count) => println!("  {}", &count.print(options)),
        Err(error) => println!("{}: {}", options.exe_name, error.to_string()),
    }
}
