use core::fmt::{Debug, Display, Formatter};
use std::error::Error;
use swc_common::source_map::Pos;
use swc_common::{SourceFile, Spanned};

#[derive(Debug)]
pub(crate) struct SyntaxError {
    msg: String,
}

impl SyntaxError {
    fn new(msg: &str) -> Self {
        Self {
            msg: msg.to_string(),
        }
    }
}

impl Display for SyntaxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl Error for SyntaxError {}

fn get_column_index_of_pos(text: &str, pos: usize) -> usize {
    let line_start_byte_pos = get_line_start_byte_pos(text, pos);
    text[line_start_byte_pos..pos].chars().count()
}

fn get_line_start_byte_pos(text: &str, pos: usize) -> usize {
    let text_bytes = text.as_bytes();
    for i in (0..pos).rev() {
        if text_bytes.get(i) == Some(&(b'\n')) {
            return i + 1;
        }
    }

    0
}

impl SyntaxError {
    pub(crate) fn from_parser_error(
        e: &swc_ecma_parser::error::Error,
        source_file: &SourceFile,
    ) -> Self {
        let src = source_file.src.as_str();
        let lines = src
            .get(0..(e.span_lo().to_usize() + 1))
            .unwrap()
            .split('\n')
            .collect::<Vec<_>>();
        let last_line = *lines.last().unwrap();

        let line = lines.len();
        let column = get_column_index_of_pos(src, last_line.len());

        let message = format!(
            "SyntaxError: {} on line {}, column {}",
            e.kind().msg().as_ref(),
            line,
            column
        );

        Self::new(message.as_str())
    }
}
