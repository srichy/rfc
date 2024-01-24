use anyhow;
#[macro_use]
extern crate lazy_static;
use std::collections::HashMap;
use std::fs::File;
use std::mem;
use std::io::{Read, BufReader, BufRead};
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
        m.insert("2VARIABLE", w_2variable as FthAction);
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
        m.insert("S\"", w_s_quote as FthAction);
        m.insert("ABORT\"", w_abort_quote as FthAction);
        m.insert("[']", w_bracket_tick as FthAction);
        m.insert("VERBATIM", w_verbatim as FthAction);
        m.insert("HEADLESSCODE", w_headless as FthAction);

        m
    };
}

type FthAction = fn(&mut Fth) -> anyhow::Result<()>;

fn w_colon(fth: &mut Fth) -> anyhow::Result<()> {
    // FIXME: emit previous definition if one exists.
    // Or not.  Maybe compile a list of word defns and
    // then iterate over that after parsing.  That would
    // all simple generation of separate sections if a
    // target type requires it.  TBD.
    fth.is_compiling = true;
    fth.input_mgr.skip_ws()?;
    let w_to_be_defined = fth.input_mgr.word()?;
    println!("@@@ WORD {w_to_be_defined:?}");

    Ok(())
}

fn w_semicolon(fth: &mut Fth) -> anyhow::Result<()> {
    fth.is_compiling = false;
    Ok(())
}

fn w_code(fth: &mut Fth) -> anyhow::Result<()> {
    fth.input_mgr.skip_ws()?;
    let w_to_be_defined = fth.input_mgr.word()?;
    println!("@@@ CODE WORD: {w_to_be_defined:?}");
    // FIXME: define a "word" with a name for the
    // dictionary.  This is different than for
    // "VERBATIM", as those instances are unnamed.
    // FIXME: collect all lines until "END-CODE"
    let code_lines = fth.input_mgr.lines_until("END-CODE");
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

fn w_verbatim(fth: &mut Fth) -> anyhow::Result<()> {
    println!("@@@ VERBATIM");
    let code_lines = fth.input_mgr.lines_until("END-VERBATIM");
    Ok(())
}

fn w_headless(fth: &mut Fth) -> anyhow::Result<()> {
    println!("@@@ HEADLESSCODE");
    let code_lines = fth.input_mgr.lines_until("END-CODE");
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
                if needs_underscore || (c != '_' && !c.is_ascii_alphabetic()) {
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
    is_compiling: bool,
    data_stack: Vec<i64>,
}

impl Fth {
    pub fn new() -> Fth {
        Fth {
            input_mgr: InputMgr::new(),
            is_compiling: false,
            data_stack: Vec::new(),
        }
    }

    fn do_literal(&mut self, n: i64) {
        println!("@@@======== FIXME: do_literal '{n}'");
    }

    fn do_number(&mut self, n: i64) {
        if self.is_compiling {
            self.do_literal(n);
        } else {
            self.data_stack.push(n);
        }
    }

    fn compile_word(&mut self, w: String) {
        println!("@@@======== FIXME: compile_word '{w}'");
    }

    pub fn interpret(&mut self, in_file: &str) -> anyhow::Result<()> {
        self.input_mgr.open_file(in_file)?;

        loop {
            self.input_mgr.skip_ws()?;
            let w = self.input_mgr.word()?;
            match w {
                None => break,
                Some(w) => {
                    let upper_w = w.to_uppercase();
                    match ACTIVE_WORDS.get(&*upper_w) {
                        None => {
                            if w.starts_with("0x") {
                                let n = i64::from_str_radix(&w[2..], 16)?;
                                self.do_number(n);
                            } else if w.starts_with("0b") {
                                let n = i64::from_str_radix(&w[2..], 2)?;
                                self.do_number(n);
                            } else {
                                match i64::from_str_radix(&*w, 10) {
                                    Ok(n) => {
                                        self.do_number(n);
                                    }
                                    Err(_) => {
                                        if self.is_compiling {
                                            self.compile_word(w);
                                        } else {
                                            // FIXME
                                            // println!("*** FIXME: handle bad immediate!")
                                        }
                                    }
                                }
                            }
                        }
                        Some(action) => {
                            action(self)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

struct InputMgr {
    input_readers: Vec<BufReader<File>>,
    last_chars: String,
}

impl InputMgr {
    pub fn new() -> Self {
        InputMgr {
            input_readers: Vec::new(),
            last_chars: String::new(),
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

    fn next_char(&mut self) -> anyhow::Result<Option<char>> {
        let mut chars = self.last_chars.chars();
        if let Some(r_char) = chars.next() {
            self.last_chars = chars.as_str().to_string();
            return Ok(Some(r_char));
        }
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
                        return Ok(Some(char::from(read_buf[0])))
                    }
                }
            }
        }
    }

    pub fn lines_until(&mut self, end_marker: &str) -> anyhow::Result<Vec<String>> {
        let mut r_lines = Vec::new();

        if self.last_chars.len() > 0 {
            // FIXME: figure out how to use string.take()
            let mut new_str = String::new();
            mem::swap(&mut new_str, &mut self.last_chars);
            r_lines.push(new_str);
        }

        'read_loop: loop {
            match self.input_readers.last_mut() {
                None => {
                    return Ok(r_lines);
                }
                Some(reader) => {
                    let mut read_buf = String::new();
                    let n_read = reader.read_line(&mut read_buf)?;
                    if n_read == 0 {
                        self.close_current()?;
                        continue 'read_loop;
                    } else {
                        if read_buf.starts_with(end_marker) {
                            return Ok(r_lines);
                        }
                        r_lines.push(read_buf);
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
                    self.last_chars = String::from(c);
                    return Ok(());
                }
            };
        }
    }

    pub fn str_by(&mut self, break_when: impl Fn(char) -> bool) -> anyhow::Result<Option<String>> {
        let mut r_str = String::new();

        loop {
            match self.next_char()? {
                None => {
                    if r_str.len() > 0 {
                        return Ok(Some(r_str));
                    } else {
                        return Ok(None);
                    }
                }
                Some(c) => {
                    if break_when(c) {
                        // In this application (Forth), there is no need to
                        // "unread" the delimiter char.
                        return Ok(Some(r_str));
                    }
                    r_str.push(c);
                }
            }
        }
    }

    pub fn word(&mut self) -> anyhow::Result<Option<String>> {
        self.str_by(|c: char| c.is_whitespace())
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Args::parse();
    let mut fth = Fth::new();
    fth.interpret(&cli.filename)?;

    Ok(())
}
