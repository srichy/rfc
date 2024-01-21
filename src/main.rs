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
    last_char: Option<u8>,
}

impl InputMgr {
    pub fn new() -> Self {
        InputMgr {
            input_readers: Vec::new(),
            last_char: None,
        }
    }

    pub fn open_file(&mut self, filename: &str) -> anyhow::Result<()> {
        let f = File::open(filename)?;
        let r = BufReader::new(f);
        self.input_readers.push(r);
        Ok(())
    }

    pub fn close_current(&mut self) -> anyhow::Result<bool> {
        match self.input_readers.pop() {
            None => Ok(false),
            Some(_) => {
                Ok(true)
            }
        }
    }

    fn next_char(&mut self) -> anyhow::Result<Option<u8>> {
        match self.last_char.take() {
            Some(c) => Ok(Some(c)),
            None => {
                'read_loop: loop {
                    match self.input_readers.last_mut() {
                        None => return Ok(None),
                        Some(reader) => {
                            let mut read_buf = [0; 1];
                            let n_read = reader.read(&mut read_buf)?;
                            if n_read == 0 {
                                self.close_current()?;
                                continue 'read_loop;
                            } else {
                                return Ok(Some(read_buf[0]))
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn skip_ws(&mut self) -> anyhow::Result<()> {
        loop {
            match self.next_char()? {
                None => return Ok(()),
                Some(c) => {
                    if (c as char).is_whitespace() {
                        continue
                    }
                    self.last_char = Some(c);
                    return Ok(());
                }
            };
        }
    }

    pub fn str_by(&mut self, break_when: impl Fn(u8) -> bool) -> anyhow::Result<Option<String>> {
        let mut buf = Vec::new();

        loop {
            match self.next_char()? {
                None => {
                    if buf.len() > 0 {
                        let r_str = String::from_utf8(buf)?;
                        return Ok(Some(r_str));
                    } else {
                        return Ok(None);
                    }
                }
                Some(c) => {
                    if break_when(c) {
                        // In this application (Forth), there is no need to
                        // "unread" the delimiter char.
                        let r_str = String::from_utf8(buf)?;
                        return Ok(Some(r_str));
                    }
                    buf.push(c);
                }
            }
        }
    }

    pub fn word(&mut self) -> anyhow::Result<Option<String>> {
        self.str_by(|c: u8| (c as char).is_whitespace())
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Args::parse();
    let mut im = InputMgr::new();
    im.open_file(&cli.filename)?;

    loop {
        im.skip_ws()?;
        let w = im.word()?;
        match w {
            None => break,
            Some(w) => println!("@@@ {w}"),
        }
    }

    Ok(())
}
