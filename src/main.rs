use anyhow;
#[macro_use]
extern crate lazy_static;
use std::collections::{HashMap, HashSet};
use clap::{Parser, ValueEnum};

mod input_mgr;
use input_mgr::InputMgr;

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Args {
    #[arg(short, long, value_enum)]
    arch: Arch,

    #[arg(short, long)]
    defines: Option<String>,

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
        m.insert("XALLOT", w_allot as FthAction);
        m.insert("BEGIN", w_begin as FthAction);
        m.insert("WHILE", w_while as FthAction);
        m.insert("REPEAT", w_repeat as FthAction);
        m.insert("UNTIL", w_until as FthAction);
        m.insert("AGAIN", w_again as FthAction);
        m.insert("IF", w_if as FthAction);
        m.insert("THEN", w_then as FthAction);
        m.insert("DO", w_do as FthAction);
        m.insert("LEAVE", w_leave as FthAction);
        m.insert("LOOP", w_loop as FthAction);
        m.insert("+LOOP", w_plus_loop as FthAction);
        m.insert("ELSE", w_else as FthAction);
        m.insert("IMMEDIATE", w_immediate as FthAction);
        m.insert("CASE", w_case as FthAction);
        m.insert("OF", w_of as FthAction);
        m.insert("ENDOF", w_endof as FthAction);
        m.insert("ENDCASE", w_endcase as FthAction);
        m.insert("S\"", w_s_quote as FthAction);
        m.insert(".\"", w_dot_quote as FthAction);
        m.insert("ABORT\"", w_abort_quote as FthAction);
        m.insert("[']", w_bracket_tick as FthAction);
        m.insert("VERBATIM", w_verbatim as FthAction);
        m.insert("HEADLESSCODE", w_headless as FthAction);
        m.insert("NEXT_IMMEDIATE", w_next_immediate as FthAction);
        m.insert("[DEFINED]", w_is_defined as FthAction);
        m.insert("[IF]", w_comp_if as FthAction);
        m.insert("[ELSE]", w_comp_else as FthAction);
        m.insert("[THEN]", w_comp_then as FthAction);
        m.insert("INCLUDE", w_include as FthAction);

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
    fth.close_definition();

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
    fth.close_definition();

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

fn w_allot(fth: &mut Fth) -> anyhow::Result<()> {
    match fth.data_stack.pop() {
        None => panic!("Stack underflow for ALLOT"),
        Some(v) => fth.allot_space(v.try_into().expect("Bad numerical format for u64")),
    }

    Ok(())
}

fn w_begin(fth: &mut Fth) -> anyhow::Result<()> {
    let lab_begin = fth.new_label();
    fth.emit_label(&lab_begin);
    fth.ctrl_other_stack.push(lab_begin);

    Ok(())
}

fn w_while(fth: &mut Fth) -> anyhow::Result<()> {
    let lab_begin = fth.ctrl_other_stack.pop().expect("Missing BEGIN label at WHILE");
    let lab_end = fth.new_label();

    fth.emit_word("qbranch");
    fth.refer_to_label(&lab_end);

    fth.ctrl_other_stack.push(lab_end);
    fth.ctrl_other_stack.push(lab_begin);

    Ok(())
}

fn w_repeat(fth: &mut Fth) -> anyhow::Result<()> {
    let lab_begin = fth.ctrl_other_stack.pop().expect("Missing BEGIN label at REPEAT");
    let lab_end = fth.ctrl_other_stack.pop().expect("Missing WHILE label at REPEAT");

    fth.emit_word("branch");
    fth.refer_to_label(&lab_begin);
    fth.emit_label(&lab_end);

    Ok(())
}

fn w_until(fth: &mut Fth) -> anyhow::Result<()> {
    let lab_begin = fth.ctrl_other_stack.pop().expect("Missing BEGIN label at UNTIL");

    fth.emit_word("qbranch");
    fth.refer_to_label(&lab_begin);

    Ok(())
}

fn w_again(fth: &mut Fth) -> anyhow::Result<()> {
    let lab_begin = fth.ctrl_other_stack.pop().expect("Missing BEGIN label at AGAIN");

    fth.emit_word("branch");
    fth.refer_to_label(&lab_begin);

    Ok(())
}

fn w_if(fth: &mut Fth) -> anyhow::Result<()> {
    let label = fth.new_label();
    fth.emit_word("qbranch");
    fth.refer_to_label(&label);
    fth.ctrl_other_stack.push(label);

    Ok(())
}

fn w_else(fth: &mut Fth) -> anyhow::Result<()> {
    let head_label = fth.ctrl_other_stack.pop().expect("Missing IF for ELSE");
    let else_label = fth.new_label();
    fth.emit_word("branch");
    fth.refer_to_label(&else_label);
    fth.ctrl_other_stack.push(else_label);
    fth.emit_label(&head_label);

    Ok(())
}

fn w_then(fth: &mut Fth) -> anyhow::Result<()> {
    let label = fth.ctrl_other_stack.pop().expect("Missing IF/ELSE for THEN");
    fth.emit_label(&label);

    Ok(())
}

fn w_do(fth: &mut Fth) -> anyhow::Result<()> {
    let backward = fth.new_label();
    let forward = fth.new_label();
    fth.emit_word("2to_r");
    fth.emit_label(&backward);
    fth.ctrl_do_stack.push(backward);
    fth.ctrl_do_stack.push(forward);

    Ok(())
}

fn w_leave(fth: &mut Fth) -> anyhow::Result<()> {
    let label = fth.ctrl_do_stack.last().expect("Missing DO for LOOP").clone();
    fth.emit_word("branch");
    fth.refer_to_label(&label);

    Ok(())
}

fn  w_loop(fth: &mut Fth) -> anyhow::Result<()> {
    let forward = fth.ctrl_do_stack.pop().expect("Missing DO for LOOP");
    let backward = fth.ctrl_do_stack.pop().expect("Missing DO for LOOP");
    fth.emit_word("do_loop1");
    fth.refer_to_label(&backward);
    fth.emit_label(&forward);
    fth.emit_word("unloop");

    Ok(())
}

fn  w_plus_loop(fth: &mut Fth) -> anyhow::Result<()> {
    let forward = fth.ctrl_do_stack.pop().expect("Missing DO for LOOP");
    let backward = fth.ctrl_do_stack.pop().expect("Missing DO for LOOP");
    fth.emit_word("do_plus_loop1");
    fth.refer_to_label(&backward);
    fth.emit_label(&forward);
    fth.emit_word("unloop");

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

fn w_immediate(_fth: &mut Fth) -> anyhow::Result<()> {
    panic!("FIXME: make a decision about word caching or not, please.");

    //Ok(())
}

fn w_case(fth: &mut Fth) -> anyhow::Result<()> {
    let label = fth.new_label();
    fth.ctrl_other_stack.push(label);

    Ok(())
}

fn w_of(fth: &mut Fth) -> anyhow::Result<()> {
    let lab_skip = fth.new_label();
    fth.emit_word("over");
    fth.emit_word("equals");
    fth.emit_word("qbranch");
    fth.refer_to_label(&lab_skip);
    fth.emit_word("drop");

    fth.ctrl_other_stack.push(lab_skip);
    Ok(())
}

fn w_endof(fth: &mut Fth) -> anyhow::Result<()> {
    let lab_skip = fth.ctrl_other_stack.pop().expect("Missing OF for ENDOF");
    let lab_end = fth.ctrl_other_stack.last().expect("Missing CASE for ENDOF");
    let lab_end = lab_end.clone();

    fth.emit_word("branch");
    fth.refer_to_label(&lab_end);
    fth.emit_label(&lab_skip);

    Ok(())
}

fn w_endcase(fth: &mut Fth) -> anyhow::Result<()> {
    let lab_end = fth.ctrl_other_stack.pop().expect("Missing ENDOF for ENDCASE");
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
    fth.refer_to_label(&branch_target);
    fth.emit_label(&string_loc);
    fth.do_string_literal(&term_str);
    fth.emit_label(&branch_target);
    fth.emit_word("lit");
    fth.refer_to_label(&string_loc);
    fth.do_literal(term_str.len() as i64);

    Ok(())
}

fn w_dot_quote(fth: &mut Fth) -> anyhow::Result<()> {
    fth.input_mgr.skip_ws()?;
    let term_str = fth.input_mgr.str_by(|c: char| c == '"')?;
    let term_str = term_str.expect("Unterminated string for '.\"'");
    let branch_target = fth.new_label();
    let string_loc = fth.new_label();
    fth.emit_word("branch");
    fth.refer_to_label(&branch_target);
    fth.emit_label(&string_loc);
    fth.do_string_literal(&term_str);
    fth.emit_label(&branch_target);
    fth.emit_word("lit");
    fth.refer_to_label(&string_loc);
    fth.do_literal(term_str.len() as i64);
    fth.emit_word("type");

    Ok(())
}

fn w_abort_quote(fth: &mut Fth) -> anyhow::Result<()> {
    fth.input_mgr.skip_ws()?;
    let term_str = fth.input_mgr.str_by(|c: char| c == '"')?;
    let term_str = term_str.expect("Unterminated string for 'abort\"'");
    let cont_target = fth.new_label();
    let abort_target = fth.new_label();
    let string_loc = fth.new_label();
    fth.emit_word("qbranch");
    fth.refer_to_label(&cont_target);
    fth.emit_word("branch");
    fth.refer_to_label(&abort_target);
    fth.emit_label(&string_loc);
    fth.do_string_literal(&term_str);
    fth.emit_label(&abort_target);
    fth.emit_word("lit");
    fth.refer_to_label(&string_loc);
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
    fth.emit_word("lit");
    fth.emit_word(&w);

    Ok(())
}

fn w_next_immediate(fth: &mut Fth) -> anyhow::Result<()> {
    fth.next_is_immediate = true;

    Ok(())
}

fn w_is_defined(fth: &mut Fth) -> anyhow::Result<()> {
    fth.input_mgr.skip_ws()?;
    let def_name = fth.input_mgr.word()?;
    let def_name = def_name.expect("EOF after [defined]!");
    if fth.defines.contains(&def_name) {
        fth.data_stack.push(-1);
    } else {
        fth.data_stack.push(0);
    }

    Ok(())
}

/* [IF]
 * This will set the "skip state".  Checks for [ELSE] and [THEN]
 * are special.
 */
fn w_comp_if(fth: &mut Fth) -> anyhow::Result<()> {
    let should_compile = match fth.data_stack.pop() {
        None => panic!("Stack underflow for [IF]"),
        Some(v) => v != 0
    };

    if should_compile {
        fth.skip_stack.push(CondCompileState::CompileUntilElse);
    } else {
        fth.skip_stack.push(CondCompileState::SkipUntilElse);
    }
    Ok(())
}

fn w_comp_else(_fth: &mut Fth) -> anyhow::Result<()> {

    panic!("*** Internal error: [ELSE] action reached!");
}

fn w_comp_then(_fth: &mut Fth) -> anyhow::Result<()> {
    panic!("*** Internal error: [THEN] action reached!");
}

fn w_include(fth: &mut Fth) -> anyhow::Result<()> {
    fth.input_mgr.skip_ws()?;
    let file_name = fth.input_mgr.word()?;
    let file_name = file_name.expect("EOF after include!");
    fth.input_mgr.open_file(&file_name)?;

    Ok(())
}

fn word_to_symbol(word_string: &str) -> String {
    let mut result = String::from("w_");
    let mut needs_underscore = false;

    for c in word_string.chars() {
        if needs_underscore {
            result.push('_');
            needs_underscore = false;
        }
        match SYMLINKAGE.get(&c) {
            None => {
                result.push(c);
            }
            Some(map_value) => {
                result.push_str(map_value);
                needs_underscore = true;
            }
        }
    }
    result
}

enum EscapeMethod {
    Backslash,
    Double,
}

fn escape_quotes(method: EscapeMethod, w: &str) -> String {
    let mut result = String::new();

    for c in w.chars() {
        if c == '"' {
            match method {
                EscapeMethod::Backslash => result.push('\\'),
                EscapeMethod::Double => result.push('"'),
            }
            result.push(c);
        } else {
            result.push(c);
        }
    }
    result
}

pub trait FthGen {
    fn prolog(&mut self);
    fn do_literal(&mut self, n: i64);
    fn do_string_literal(&mut self, s: &str);
    fn create_word(&mut self, w: &str, is_immediate: bool);
    fn create_code(&mut self, w: &str, is_immediate: bool);
    fn close_definition(&mut self);
    fn emit_word(&mut self, w: &str);
    fn emit_lines(&mut self, lines: Vec<String>);
    fn refer_to_label(&mut self, w: &str);
    fn emit_label(&mut self, l: &str);
    fn create_constant(&mut self, name: &str, val: i64);
    fn create_variable(&mut self, name: &str, size: u8);
    fn allot_space(&mut self, size: u64);
    fn epilog(&mut self);
}

struct AttGen {
    _is_compiling: bool,
    last_dict_entry: String,
}

impl AttGen {
    fn new() -> Self {
        AttGen {
            _is_compiling: false,
            last_dict_entry: String::from("0"),
        }
    }
}

impl FthGen for AttGen {
    fn prolog(&mut self) {
    }

    fn do_literal(&mut self, n: i64) {
        println!("    .int w_lit");
        let l = n as i32;
        println!("    .int {l}");
    }

    fn do_string_literal(&mut self, s: &str) {
        println!("    .ascii \"{s}\"");
    }

    fn create_word(&mut self, w: &str, is_immediate: bool) {
        let word_sym = word_to_symbol(&w);
        let word_len = w.len();
        let w = escape_quotes(EscapeMethod::Backslash, w);
        let flags:u8 = if is_immediate { 1 } else { 0 };
        println!("    HIGH_W {word_sym} {word_len} \"{w}\" flgs={flags}");
        self.last_dict_entry = word_sym.clone();
    }

    fn create_code(&mut self, w: &str, is_immediate: bool) {
        let word_sym = word_to_symbol(&w);
        let word_len = w.len();
        let w = escape_quotes(EscapeMethod::Backslash, w);
        let flags:u8 = if is_immediate { 1 } else { 0 };
        println!("    CODE_W {word_sym} {word_len} \"{w}\" flgs={flags}");
        self.last_dict_entry = word_sym.clone();
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

    fn refer_to_label(&mut self, w: &str) {
        println!("    .int {w}");
    }

    fn emit_label(&mut self, l: &str) {
        println!("{l}:");
    }

    fn create_constant(&mut self, name: &str, val: i64) {
        let name_sym = word_to_symbol(&name);
        let name_len = name.len();
        let name = escape_quotes(EscapeMethod::Backslash, name);
        let const_val = val as i32;
        println!("    HIGH_W {name_sym} {name_len} \"{name}\" act=w_do_const");
        println!("    .int {const_val}");
        self.last_dict_entry = name_sym.clone();
    }

    fn create_variable(&mut self, name: &str, size: u8) {
        let name_sym = word_to_symbol(&name);
        let name_len = name.len();
        let name = escape_quotes(EscapeMethod::Backslash, name);
        println!("    HIGH_W {name_sym} {name_len} \"{name}\" act=w_do_var");
        for _ in 0..size {
            println!("    .int 0");
        }
        self.last_dict_entry = name_sym.clone();
    }

    fn allot_space(&mut self, size: u64) {
        println!("    .space {size}");
    }

    fn epilog(&mut self) {
        let de = &self.last_dict_entry;
        println!("dict_head: .int dict_{de}");
    }
}

struct Ca6502 {
    _is_compiling: bool,
    last_dict_entry: String,
}

impl Ca6502 {
    fn new() -> Self {
        Ca6502 {
            _is_compiling: false,
            last_dict_entry: String::from("0"),
        }
    }
}

impl FthGen for Ca6502 {
    fn prolog(&mut self) {
    }

    fn do_literal(&mut self, n: i64) {
        println!("    .word w_lit.cfa");
        let l = n as i16;
        if l < 0 {
            println!("    .sint {l}");
        } else {
            println!("    .word {l}");
        }
    }

    fn do_string_literal(&mut self, s: &str) {
        println!("    .text \"{s}\"");
    }

    fn create_word(&mut self, w: &str, is_immediate: bool) {
        let word_sym = word_to_symbol(&w);
        let word_len = w.len();
        let mut w = escape_quotes(EscapeMethod::Double, w);
        let flags:u8 = if is_immediate { 1 } else { 0 };
        w.make_ascii_uppercase();
        let last_ref = &self.last_dict_entry;
        println!("{word_sym}    .HIGH_W {word_len}, \"{w}\", , {flags}, {last_ref}");
        println!("  .block");
        self.last_dict_entry = word_sym.clone();
    }

    fn create_code(&mut self, w: &str, is_immediate: bool) {
        let word_sym = word_to_symbol(&w);
        let word_len = w.len();
        let mut w = escape_quotes(EscapeMethod::Double, w);
        let flags:u8 = if is_immediate { 1 } else { 0 };
        w.make_ascii_uppercase();
        let last_ref = &self.last_dict_entry;
        println!("{word_sym}    .CODE_W {word_len}, \"{w}\", {flags}, {last_ref}");
        println!("  .block");
        self.last_dict_entry = word_sym.clone();
    }

    fn close_definition(&mut self) {
        println!("  .endblock");
    }

    fn emit_word(&mut self, w: &str) {
        let word_sym = word_to_symbol(&w);
        println!("    .addr {word_sym}.cfa");
    }

    fn emit_lines(&mut self, lines: Vec<String>) {
        for l in lines {
            print!("{l}");
        }
    }

    fn refer_to_label(&mut self, w: &str) {
        println!("    .addr {w}");
    }

    fn emit_label(&mut self, l: &str) {
        println!("{l}");
    }

    fn create_constant(&mut self, name: &str, val: i64) {
        let name_sym = word_to_symbol(&name);
        let name_len = name.len();
        let mut name = escape_quotes(EscapeMethod::Double, name);
        let const_val = val as i32;
        name.make_ascii_uppercase();
        let last_ref = &self.last_dict_entry;
        println!("{name_sym}    .HIGH_W {name_len}, \"{name}\", w_const, , {last_ref}");
        if const_val < 0 {
            println!("    .sint {const_val}");
        } else {
            println!("    .word {const_val}");
        }
        self.last_dict_entry = name_sym.clone();
    }

    fn create_variable(&mut self, name: &str, size: u8) {
        let name_sym = word_to_symbol(&name);
        let name_len = name.len();
        let mut name = escape_quotes(EscapeMethod::Double, name);
        name.make_ascii_uppercase();
        let last_ref = &self.last_dict_entry;
        println!("{name_sym}    .HIGH_W {name_len}, \"{name}\", w_var, , {last_ref}");
        for _ in 0..size {
            println!("    .word 0");
        }
        self.last_dict_entry = name_sym.clone();
    }

    fn allot_space(&mut self, size: u64) {
        println!("    .fill {size}");
    }

    fn epilog(&mut self) {
        let de = &self.last_dict_entry;
        println!("dict_head .addr {de}");
    }
}

#[derive(PartialEq, Copy, Clone)]
enum CondCompileState {
    Skipping,
    SkipUntilElse,
    CompileUntilElse,
}

struct Fth {
    gen: Box<dyn FthGen>,
    defines: HashSet<String>,
    input_mgr: InputMgr,
    is_compiling: bool,
    skip_stack: Vec<CondCompileState>,
    data_stack: Vec<i64>,
    ctrl_do_stack: Vec<String>,
    ctrl_other_stack: Vec<String>,
    next_label: u32,
    next_is_immediate: bool,
}

impl Fth {
    pub fn new(arch: Arch, defines: Option<String>) -> Fth {
        let g: Box<dyn FthGen> = match arch {
            Arch::C => panic!("C not supported yet"),
            Arch::AttAsm32 => Box::new(AttGen::new()),
            Arch::Ca6502 => Box::new(Ca6502::new()),
        };
        let def_strings = defines.unwrap_or(String::new());
        let def_strings: Vec<String> =
            def_strings.split_terminator(',').map(|sp| String::from(sp)).collect();
        let mut defines_set: HashSet<String> = HashSet::new();
        for s in def_strings {
            defines_set.insert(s);
        }

        Fth {
            gen: g,
            defines: defines_set,
            input_mgr: InputMgr::new(),
            is_compiling: false,
            skip_stack: Vec::new(),
            data_stack: Vec::new(),
            ctrl_do_stack: Vec::new(),
            ctrl_other_stack: Vec::new(),
            next_label: 1,
            next_is_immediate: false,
        }
    }

    fn do_skip(&mut self, w: &str) -> bool {
        let w = w.to_uppercase();

        if self.skip_stack.is_empty() {
            if w == "[THEN]" || w == "[ELSE]" {
                panic!("Encountered {w} without matching [IF]");
            }
            return false;
        }

        let cur_action = *self.skip_stack.last().unwrap();
        let is_skipping = cur_action == CondCompileState::Skipping ||
            cur_action == CondCompileState::SkipUntilElse;

        if w == "[ELSE]" {
            match cur_action {
                CondCompileState::SkipUntilElse => {
                    self.skip_stack.pop();
                    self.skip_stack.push(CondCompileState::CompileUntilElse);
                }
                CondCompileState::CompileUntilElse => {
                    self.skip_stack.pop();
                    self.skip_stack.push(CondCompileState::SkipUntilElse);
                }
                CondCompileState::Skipping => {
                    // Do nothing.  Keep skip nesting constant here, but
                    // also keep on skipping
                }
            }
            return true
        }

        if w == "[THEN]" {
            self.skip_stack.pop();
            return true
        }

        // Hitting [IF] (or [THEN]) while skipping is special because we have to
        // track nesting.  And [IF] is handled if _not_ skipping via its action.
        if is_skipping {
            if w == "[IF]" {
                self.skip_stack.push(CondCompileState::Skipping);
                return true;
            }
        }

        is_skipping
    }

    fn new_label(&mut self) -> String {
        let label_index = self.next_label;
        let label_str = format!("_L{label_index:03}");
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

    fn refer_to_label(&mut self, w: &str) {
        self.gen.refer_to_label(w);
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

    fn allot_space(&mut self, size: u64) {
        self.gen.allot_space(size);
    }

    fn emit_lines(&mut self, lines: Vec<String>) {
        self.gen.emit_lines(lines);
    }

    pub fn interpret(&mut self, in_file: &str) -> anyhow::Result<()> {
        self.input_mgr.open_file(in_file)?;

        self.gen.prolog();
        loop {
            self.input_mgr.skip_ws()?;
            let w = self.input_mgr.word()?;
            match w {
                None => break,
                Some(w) => {

                    if self.do_skip(&w) {
                        // [IF], [ELSE], [THEN] are "special"
                        continue;
                    }

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
                                            println!("*** FIXME: handle bad immediate: '{w}'");
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
        self.gen.epilog();

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Args::parse();
    let mut fth = Fth::new(cli.arch, cli.defines);
    fth.interpret(&cli.filename)?;

    Ok(())
}
