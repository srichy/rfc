use anyhow;
#[macro_use]
extern crate lazy_static;
use std::collections::HashMap;
use clap::{Parser, ValueEnum};

mod input_mgr;
use input_mgr::InputMgr;

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
    AttAsm32,
    Ca6502,
}

type FthAction = fn(&mut Fth) -> anyhow::Result<()>;

lazy_static! {
    static ref SYMLINKAGE: HashMap<char, &'static str> = {
        let mut m = HashMap::new();

        m.insert(':', "colon");
        m.insert(';', "semicolon");
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
        m.insert('<', "from");
        m.insert('>', "to");
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
        m.insert("UNTIL", w_until as FthAction);
        m.insert("AGAIN", w_again as FthAction);
        m.insert("IF", w_if as FthAction);
        m.insert("THEN", w_then as FthAction);
        m.insert("DO", w_do as FthAction);
        m.insert("LOOP", w_loop as FthAction);
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
        m.insert("NEXT-IMMEDIATE", w_next_immediate as FthAction);

        m
    };
}

fn w_colon(fth: &mut Fth) -> anyhow::Result<()> {
    fth.is_compiling = true;
    fth.input_mgr.skip_ws()?;
    let w_to_be_defined = fth.input_mgr.word()?;
    let w_to_be_defined = w_to_be_defined.expect("EOF after colon!");
    let next_is_immediate = fth.next_is_immediate;
    fth.next_is_immediate = false;
    fth.create_word(&w_to_be_defined, next_is_immediate);

    Ok(())
}

fn w_semicolon(fth: &mut Fth) -> anyhow::Result<()> {
    fth.is_compiling = false;
    fth.emit_word("exit");

    Ok(())
}

fn w_code(fth: &mut Fth) -> anyhow::Result<()> {
    fth.input_mgr.skip_ws()?;
    let w_to_be_defined = fth.input_mgr.word()?;
    let w_to_be_defined = w_to_be_defined.expect("EOF while defining CODE");
    let next_is_immediate = fth.next_is_immediate;
    fth.next_is_immediate = false;
    fth.create_code(&w_to_be_defined, next_is_immediate);
    let mut code_lines = fth.input_mgr.lines_until("END-CODE")?;
    code_lines.push("    NEXT\n".to_string());
    fth.emit_lines(code_lines);

    Ok(())
}

fn w_paren(fth: &mut Fth) -> anyhow::Result<()> {
    let _ = fth.input_mgr.str_by(|c: char| c == ')')?;

    Ok(())
}

fn w_constant(fth: &mut Fth) -> anyhow::Result<()> {
    fth.input_mgr.skip_ws()?;
    let constant_name = fth.input_mgr.word()?;
    let constant_name = constant_name.expect("EOF while defining a CONSTANT");
    match fth.data_stack.pop() {
        None => panic!("Stack underflow for CONSTANT '{constant_name}"),
        Some(v) => fth.create_constant(&constant_name, v),
    }

    Ok(())
}

fn w_variable(fth: &mut Fth) -> anyhow::Result<()> {
    fth.input_mgr.skip_ws()?;
    let variable_name = fth.input_mgr.word()?;
    let variable_name = variable_name.expect("EOF while defining a VARIABLE");
    fth.create_variable(&variable_name, 1);

    Ok(())
}

fn w_2variable(fth: &mut Fth) -> anyhow::Result<()> {
    fth.input_mgr.skip_ws()?;
    let variable_name = fth.input_mgr.word()?;
    let variable_name = variable_name.expect("EOF while defining a 2VARIABLE");
    fth.create_variable(&variable_name, 2);

    Ok(())
}

fn w_begin(fth: &mut Fth) -> anyhow::Result<()> {
    let lab_begin = fth.new_label();
    fth.emit_label(&lab_begin);
    fth.ctrl_stack.push(lab_begin);

    Ok(())
}

fn w_while(fth: &mut Fth) -> anyhow::Result<()> {
    let lab_begin = fth.ctrl_stack.pop().expect("Missing BEGIN label at WHILE");
    let lab_end = fth.new_label();

    fth.emit_word("qbranch");
    fth.compute_label(&lab_end);

    fth.ctrl_stack.push(lab_end);
    fth.ctrl_stack.push(lab_begin);

    Ok(())
}

fn w_repeat(fth: &mut Fth) -> anyhow::Result<()> {
    let lab_begin = fth.ctrl_stack.pop().expect("Missing BEGIN label at REPEAT");
    let lab_end = fth.ctrl_stack.pop().expect("Missing WHILE label at REPEAT");

    fth.emit_word("branch");
    fth.compute_label(&lab_begin);
    fth.emit_label(&lab_end);

    Ok(())
}

fn w_until(fth: &mut Fth) -> anyhow::Result<()> {
    let lab_begin = fth.ctrl_stack.pop().expect("Missing BEGIN label at REPEAT");

    fth.emit_word("qbranch");
    fth.compute_label(&lab_begin);

    Ok(())
}

fn w_again(fth: &mut Fth) -> anyhow::Result<()> {
    let lab_begin = fth.ctrl_stack.pop().expect("Missing BEGIN label at REPEAT");

    fth.emit_word("branch");
    fth.compute_label(&lab_begin);

    Ok(())
}

fn w_if(fth: &mut Fth) -> anyhow::Result<()> {
    let label = fth.new_label();
    fth.emit_word("qbranch");
    fth.compute_label(&label);
    fth.ctrl_stack.push(label);

    Ok(())
}

fn w_else(fth: &mut Fth) -> anyhow::Result<()> {
    let head_label = fth.ctrl_stack.pop().expect("Missing IF for ELSE");
    let else_label = fth.new_label();
    fth.emit_word("branch");
    fth.compute_label(&else_label);
    fth.ctrl_stack.push(else_label);
    fth.emit_label(&head_label);

    Ok(())
}

fn w_then(fth: &mut Fth) -> anyhow::Result<()> {
    let label = fth.ctrl_stack.pop().expect("Missing IF/ELSE for THEN");
    fth.emit_label(&label);

    Ok(())
}

fn w_do(fth: &mut Fth) -> anyhow::Result<()> {
    let label = fth.new_label();
    fth.emit_word("_2_to_r");
    fth.compute_label(&label);
    fth.ctrl_stack.push(label);

    Ok(())
}

fn  w_loop(fth: &mut Fth) -> anyhow::Result<()> {
    let label = fth.ctrl_stack.pop().expect("Missing DO for LOOP");
    fth.emit_word("do_loop");
    fth.compute_label(&label);

    Ok(())
}

fn w_verbatim(fth: &mut Fth) -> anyhow::Result<()> {
    let code_lines = fth.input_mgr.lines_until("END-VERBATIM")?;
    fth.emit_lines(code_lines);

    Ok(())
}

fn w_headless(fth: &mut Fth) -> anyhow::Result<()> {
    let code_lines = fth.input_mgr.lines_until("END-CODE")?;
    fth.emit_lines(code_lines);

    Ok(())
}

fn w_immediate(fth: &mut Fth) -> anyhow::Result<()> {
    panic!("FIXME: make a decision about word caching or not, please.");

    //Ok(())
}

fn w_case(fth: &mut Fth) -> anyhow::Result<()> {
    let label = fth.new_label();
    fth.ctrl_stack.push(label);

    Ok(())
}

fn w_of(fth: &mut Fth) -> anyhow::Result<()> {
    let lab_skip = fth.new_label();
    fth.emit_word("over");
    fth.emit_word("equals");
    fth.emit_word("qbranch");
    fth.compute_label(&lab_skip);
    fth.emit_word("drop");

    fth.ctrl_stack.push(lab_skip);
    Ok(())
}

fn w_endof(fth: &mut Fth) -> anyhow::Result<()> {
    let lab_skip = fth.ctrl_stack.pop().expect("Missing OF for ENDOF");
    let lab_end = fth.ctrl_stack.last().expect("Missing CASE for ENDOF");
    let lab_end = lab_end.clone();

    fth.emit_word("branch");
    fth.compute_label(&lab_end);
    fth.emit_label(&lab_skip);

    Ok(())
}

fn w_endcase(fth: &mut Fth) -> anyhow::Result<()> {
    let lab_end = fth.ctrl_stack.pop().expect("Missing ENDOF for ENDCASE");
    fth.emit_word("drop");
    fth.emit_label(&lab_end);

    Ok(())
}

fn w_s_quote(fth: &mut Fth) -> anyhow::Result<()> {
    fth.input_mgr.skip_ws()?;
    let term_str = fth.input_mgr.str_by(|c: char| c == '"')?;
    let term_str = term_str.expect("Unterminated string for 's\"'");
    let branch_target = fth.new_label();
    let string_loc = fth.new_label();
    fth.emit_word("branch");
    fth.compute_label(&branch_target);
    fth.emit_label(&string_loc);
    fth.do_string_literal(&term_str);
    fth.emit_label(&branch_target);
    fth.emit_word("lit");
    fth.compute_label(&string_loc);
    fth.do_literal(term_str.len() as i64);

    Ok(())
}

fn w_abort_quote(fth: &mut Fth) -> anyhow::Result<()> {
    fth.input_mgr.skip_ws()?;
    let term_str = fth.input_mgr.str_by(|c: char| c == '"')?;
    let term_str = term_str.expect("Unterminated string for 's\"'");
    let cont_target = fth.new_label();
    let abort_target = fth.new_label();
    let string_loc = fth.new_label();
    fth.emit_word("qbranch");
    fth.compute_label(&cont_target);
    fth.emit_word("branch");
    fth.compute_label(&abort_target);
    fth.emit_label(&string_loc);
    fth.do_string_literal(&term_str);
    fth.emit_label(&abort_target);
    fth.emit_word("lit");
    fth.compute_label(&string_loc);
    fth.do_literal(term_str.len() as i64);
    fth.emit_word("type");
    fth.emit_word("cr");
    fth.emit_word("abort");
    fth.emit_label(&cont_target);

    Ok(())
}

fn w_bracket_tick(fth: &mut Fth) -> anyhow::Result<()> {
    fth.input_mgr.skip_ws()?;
    let w = fth.input_mgr.word()?;
    let w = w.expect("EOF in '[']'");
    let w = word_to_symbol(&w);
    fth.emit_word("lit");
    fth.emit_word(&w);

    Ok(())
}

fn w_next_immediate(fth: &mut Fth) -> anyhow::Result<()> {
    fth.next_is_immediate = true;

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

pub trait FthGen {
    fn do_literal(&mut self, n: i64);
    fn do_string_literal(&mut self, s: &str);
    fn create_word(&mut self, w: &str, is_immediate: bool);
    fn create_code(&mut self, w: &str, is_immediate: bool);
    fn close_definition(&mut self);
    fn emit_word(&mut self, w: &str);
    fn emit_lines(&mut self, lines: Vec<String>);
    fn compute_label(&mut self, w: &str);
    fn emit_label(&mut self, l: &str);
    fn create_constant(&mut self, name: &str, val: i64);
    fn create_variable(&mut self, name: &str, size: u8);
}

struct AttGen {
    is_compiling: bool,
}

impl AttGen {
    fn new() -> Self {
        AttGen {
            is_compiling: false,
        }
    }
}

impl FthGen for AttGen {
    fn do_literal(&mut self, n: i64) {
        println!("    .int lit");
        let l = n as i32;
        println!("    .int {l}");
    }

    fn do_string_literal(&mut self, s: &str) {
        println!("    .ascii \"{s}\"");
    }

    fn create_word(&mut self, w: &str, is_immediate: bool) {
        let word_sym = word_to_symbol(&w);
        let word_len = w.len();
        let flags:u8 = if is_immediate { 1 } else { 0 };
        println!("    HIGH_W {word_sym} {word_len} \"{w}\" flgs={flags}");
    }

    fn create_code(&mut self, w: &str, is_immediate: bool) {
        let word_sym = word_to_symbol(&w);
        let word_len = w.len();
        let flags:u8 = if is_immediate { 1 } else { 0 };
        println!("    CODE_W {word_sym} {word_len} \"{w}\" flgs={flags}");
    }

    fn close_definition(&mut self) {
    }

    fn emit_word(&mut self, w: &str) {
        let word_sym = word_to_symbol(&w);
        println!("    .int {word_sym}");
    }

    fn emit_lines(&mut self, lines: Vec<String>) {
        for l in lines {
            print!("{l}");
        }
    }

    fn compute_label(&mut self, w: &str) {
        println!("    .int {w}");
    }

    fn emit_label(&mut self, l: &str) {
        println!("{l}:");
    }

    fn create_constant(&mut self, name: &str, val: i64) {
        let name_sym = word_to_symbol(&name);
        let name_len = name.len();
        let const_val = val as i32;
        println!("    HIGH_W {name_sym} {name_len} \"{name}\" act=do_const");
        println!("    .int {const_val}");
    }

    fn create_variable(&mut self, name: &str, size: u8) {
        let name_sym = word_to_symbol(&name);
        let name_len = name.len();
        println!("    HIGH_W {name_sym} {name_len} \"{name}\" act=do_var");
        for _ in 0..size {
            println!("    .int 0");
        }
    }
}

struct Ca6502 {
    is_compiling: bool,
}

impl Ca6502 {
    fn new() -> Self {
        Ca6502 {
            is_compiling: false,
        }
    }
}

impl FthGen for Ca6502 {
    fn do_literal(&mut self, n: i64) {
        println!("    .word lit");
        let l = n as i16;
        println!("    .word {l}");
    }

    fn do_string_literal(&mut self, s: &str) {
        println!("    .byte \"{s}\"");
    }

    fn create_word(&mut self, w: &str, is_immediate: bool) {
        let word_sym = word_to_symbol(&w);
        let word_len = w.len();
        let flags:u8 = if is_immediate { 1 } else { 0 };
        println!("    .HIGH_W {word_sym}, {word_len}, \"{w}\", , {flags}");
    }

    fn create_code(&mut self, w: &str, is_immediate: bool) {
        let word_sym = word_to_symbol(&w);
        let word_len = w.len();
        let flags:u8 = if is_immediate { 1 } else { 0 };
        println!("    .CODE_W {word_sym}, {word_len}, \"{w}\", , {flags}");
    }

    fn close_definition(&mut self) {
    }

    fn emit_word(&mut self, w: &str) {
        let word_sym = word_to_symbol(&w);
        println!("    .word {word_sym}");
    }

    fn emit_lines(&mut self, lines: Vec<String>) {
        for l in lines {
            print!("{l}");
        }
    }

    fn compute_label(&mut self, w: &str) {
        println!("    .word {w}");
    }

    fn emit_label(&mut self, l: &str) {
        println!("{l}:");
    }

    fn create_constant(&mut self, name: &str, val: i64) {
        let name_sym = word_to_symbol(&name);
        let name_len = name.len();
        let const_val = val as i32;
        println!("    .HIGH_W {name_sym}, {name_len}, \"{name}\", do_const, ");
        println!("    .word {const_val}");
    }

    fn create_variable(&mut self, name: &str, size: u8) {
        let name_sym = word_to_symbol(&name);
        let name_len = name.len();
        println!("    .HIGH_W {name_sym}, {name_len}, \"{name}\", do_var, ");
        for _ in 0..size {
            println!("    .word 0");
        }
    }
}

struct Fth {
    gen: Box<dyn FthGen>,
    input_mgr: InputMgr,
    is_compiling: bool,
    data_stack: Vec<i64>,
    ctrl_stack: Vec<String>,
    next_label: u32,
    next_is_immediate: bool,
}

impl Fth {
    pub fn new(arch: Arch) -> Fth {
        let g: Box<dyn FthGen> = match arch {
            Arch::C => panic!("C not supported yet"),
            Arch::AttAsm32 => Box::new(AttGen::new()),
            Arch::Ca6502 => Box::new(Ca6502::new()),
        };
        Fth {
            gen: g,
            input_mgr: InputMgr::new(),
            is_compiling: false,
            data_stack: Vec::new(),
            ctrl_stack: Vec::new(),
            next_label: 1,
            next_is_immediate: false,
        }
    }

    fn new_label(&mut self) -> String {
        let label_index = self.next_label;
        let label_str = format!("L{label_index:08}");
        self.next_label += 1;
        label_str
    }

    fn do_literal(&mut self, n: i64) {
        self.gen.do_literal(n);
    }

    fn do_string_literal(&mut self, s: &str) {
        self.gen.do_string_literal(s);
    }

    fn do_number(&mut self, n: i64) {
        if self.is_compiling {
            self.gen.do_literal(n);
        } else {
            self.data_stack.push(n);
        }
    }

    fn create_word(&mut self, w: &str, is_immediate: bool) {
        self.gen.create_word(w, is_immediate);
    }

    fn create_code(&mut self, w: &str, is_immediate: bool) {
        self.gen.create_code(w, is_immediate);
    }

    fn close_definition(&mut self) {
        self.gen.close_definition();
    }

    fn emit_word(&mut self, w: &str) {
        self.gen.emit_word(w);
    }

    fn compute_label(&mut self, w: &str) {
        self.gen.compute_label(w);
    }

    fn emit_label(&mut self, l: &str) {
        self.gen.emit_label(l);
    }

    fn create_constant(&mut self, name: &str, val: i64) {
        self.gen.create_constant(name, val);
    }

    fn create_variable(&mut self, name: &str, size: u8) {
        self.gen.create_variable(name, size);
    }

    fn emit_lines(&mut self, lines: Vec<String>) {
        self.gen.emit_lines(lines);
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
                                            self.emit_word(&w);
                                        } else {
                                            // FIXME
                                            // println!("*** FIXME: handle bad immediate: '{w}'");
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

fn main() -> anyhow::Result<()> {
    let cli = Args::parse();
    let mut fth = Fth::new(cli.arch);
    fth.interpret(&cli.filename)?;

    Ok(())
}
