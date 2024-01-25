use anyhow;
use std::fs::File;
use std::mem;
use std::io::{Read, BufReader, BufRead};

pub struct InputMgr {
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

