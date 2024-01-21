use anyhow;
use std::fs::File;
use std::io::{self, Read, Write, Error, ErrorKind, Result, BufRead, BufReader};
use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    #[arg(short, long, value_enum)]
    arch: Arch,

    #[arg(help="Forth source file")]
    filename: String,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Arch {
    C,
    AttAsm,
}

struct Fth {
}

struct InputMgr {
    input_readers: Vec<BufReader<File>>,
}

impl InputMgr {
    fn new() -> Self {
        InputMgr {
            input_readers: Vec::new(),
        }
    }

    fn open_file(&mut self, filename: &str) -> anyhow::Result<()> {
        let f = File::open(filename)?;
        let r = BufReader::new(f);
        self.input_readers.push(r);
        Ok(())
    }

    fn close_current(&mut self) -> anyhow::Result<bool> {
        match self.input_readers.pop() {
            None => Ok(false),
            Some(_) => {
                Ok(true)
            }
        }
    }

    fn str_by(&mut self, break_when: impl Fn(u8) -> bool) -> anyhow::Result<String> {
        let mut buf = Vec::new();

        'read_loop: loop {
            let mut char_buf = [0; 1];

            match self.input_readers.last_mut() {
                None => return Ok(String::new()),
                Some(reader) => {
                    let read_size = reader.read(&mut char_buf)?;
                    if read_size == 0 {
                        self.close_current()?;
                        continue 'read_loop;
                    }
                    if break_when(char_buf[0]) {
                        // In this application (Forth), there is no need to
                        // "unread" the delimiter char.
                        let r_str = String::from_utf8(buf)?;
                        return Ok(r_str);
                    }
                    buf.push(char_buf[0]);
                }
            }
        }
    }

    fn word(&mut self) -> anyhow::Result<String> {
        self.str_by(|c: u8| (c as char).is_whitespace())
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Args::parse();
    let mut im = InputMgr::new();
    im.open_file(&cli.filename)?;

    loop {
        let w = im.word()?;
        if w.len() == 0 {
            break;
        }
        println!("{w}");
    }

    Ok(())
}
