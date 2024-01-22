use anyhow;
#[macro_use]
extern crate lazy_static;
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, BufReader};
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

lazy_static! {
    static ref SYMLINKAGE: HashMap<char, &'static str> = {
        let mut m = HashMap::new();

        m.insert('*', "star");
        m.insert('/', "slash");
        m.insert('\\', "backslash");
        m.insert('!', "store");
        m.insert('@', "fetch");
        m.insert('#', "pound");
        m.insert('\'', "tick");
        m.insert('`', "backtick");
        m.insert('"', "quote");
        m.insert('+', "plus");
        m.insert('-', "minus");
        m.insert(',', "comma");
        m.insert('.', "dot");
        m.insert('<', "less");
        m.insert('>', "greater");
        m.insert('=', "equals");
        m.insert('(', "open_paren");
        m.insert(')', "close_paren");
        m.insert('[', "open_square");
        m.insert(']', "close_square");
        m.insert('{', "open_brace");
        m.insert('}', "close_brace");
        m.insert('?', "question");
        m.insert('%', "percent");
        m.insert('^', "caret");
        m.insert('&', "ampersand");
        m.insert('~', "tilde");
        m.insert('|', "pipe");

        m
    };

    static ref ACTIVE_WORDS: HashMap<&'static str, FthAction> = {
        let mut m = HashMap::new();
        m.insert(":", w_colon as FthAction);
        m.insert(";", w_semicolon as FthAction);
        m.insert("CODE", w_code as FthAction);
        m.insert("(", w_paren as FthAction);
        m.insert("CONSTANT", w_constant as FthAction);
        m.insert("VARIABLE", w_variable as FthAction);
        m.insert("BEGIN", w_begin as FthAction);
        m.insert("WHILE", w_while as FthAction);
        m.insert("REPEAT", w_repeat as FthAction);
        m.insert("IF", w_if as FthAction);
        m.insert("THEN", w_then as FthAction);
        m.insert("DO", w_do as FthAction);
        m.insert("LOOP", w_loop as FthAction);
        m.insert("UNTIL", w_until as FthAction);
        m.insert("ELSE", w_else as FthAction);
        m.insert("IMMEDIATE", w_immediate as FthAction);
        m.insert("CASE", w_case as FthAction);
        m.insert("OF", w_of as FthAction);
        m.insert("ENDOF", w_endof as FthAction);
        m.insert("ENDCASE", w_endcase as FthAction);
        m.insert("2VARIABLE", w_2variable as FthAction);
        m.insert("S\"", w_s_quote as FthAction);
        m.insert("ABORT\"", w_abort_quote as FthAction);
        m.insert("[']", w_bracket_tick as FthAction);
        m
    };
}

type FthAction = fn(&mut Fth) -> anyhow::Result<()>;

fn w_colon(fth: &mut Fth) -> anyhow::Result<()> {
    Ok(())
}

fn w_semicolon(fth: &mut Fth) -> anyhow::Result<()> {
    Ok(())
}

fn w_code(fth: &mut Fth) -> anyhow::Result<()> {
    Ok(())
}

fn w_paren(fth: &mut Fth) -> anyhow::Result<()> {
    Ok(())
}

fn w_constant(fth: &mut Fth) -> anyhow::Result<()> {
    Ok(())
}

fn w_variable(fth: &mut Fth) -> anyhow::Result<()> {
    Ok(())
}

fn w_begin(fth: &mut Fth) -> anyhow::Result<()> {
    Ok(())
}

fn w_while(fth: &mut Fth) -> anyhow::Result<()> {
    Ok(())
}

fn w_repeat(fth: &mut Fth) -> anyhow::Result<()> {
    Ok(())
}

fn w_if(fth: &mut Fth) -> anyhow::Result<()> {
    Ok(())
}

fn w_then(fth: &mut Fth) -> anyhow::Result<()> {
    Ok(())
}

fn w_do(fth: &mut Fth) -> anyhow::Result<()> {
    Ok(())
}

fn  w_loop(fth: &mut Fth) -> anyhow::Result<()> {
    Ok(())
}

fn w_hcode(fth: &mut Fth) -> anyhow::Result<()> {
    Ok(())
}

fn w_until(fth: &mut Fth) -> anyhow::Result<()> {
    Ok(())
}

fn w_else(fth: &mut Fth) -> anyhow::Result<()> {
    Ok(())
}

fn w_immediate(fth: &mut Fth) -> anyhow::Result<()> {
    Ok(())
}

fn w_case(fth: &mut Fth) -> anyhow::Result<()> {
    Ok(())
}

fn w_of(fth: &mut Fth) -> anyhow::Result<()> {
    Ok(())
}

fn w_endof(fth: &mut Fth) -> anyhow::Result<()> {
    Ok(())
}

fn w_endcase(fth: &mut Fth) -> anyhow::Result<()> {
    Ok(())
}

fn w_2variable(fth: &mut Fth) -> anyhow::Result<()> {
    Ok(())
}

fn w_s_quote(fth: &mut Fth) -> anyhow::Result<()> {
    Ok(())
}

fn w_abort_quote(fth: &mut Fth) -> anyhow::Result<()> {
    Ok(())
}

fn w_bracket_tick(fth: &mut Fth) -> anyhow::Result<()> {
    Ok(())
}

fn word_to_symbol(word_string: &str) -> String {
    let mut result = String::new();
    let mut needs_underscore = false;

    for c in word_string.chars() {
        match SYMLINKAGE.get(&c) {
            None => {
                if needs_underscore {
                    result.push('_');
                    needs_underscore = false;
                }
                result.push(c);
            }
            Some(map_value) => {
                if result.len() > 0 {
                    result.push('_');
                }
                result.push_str(map_value);
                needs_underscore = true;
            }
        }
    }
    result
}

struct Fth {
    input_mgr: InputMgr,
}

impl Fth {
    pub fn new() -> Fth {
        Fth {
            input_mgr: InputMgr::new(),
        }
    }

    pub fn interpret(&mut self, in_file: &str) -> anyhow::Result<()> {
        self.input_mgr.open_file(in_file)?;

        loop {
            self.input_mgr.skip_ws()?;
            let w = self.input_mgr.word()?;
            match w {
                None => break,
                Some(w) => println!("@@@ {w}"),
            }
        }
        Ok(())
    }
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
    let mut fth = Fth::new();
    fth.interpret(&cli.filename)?;

    Ok(())
}
