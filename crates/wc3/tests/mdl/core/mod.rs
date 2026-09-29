use std::io::{self, Write};
use wc3::model::animation::{GlobalSequence, Sequence, SequenceFlags};
use wc3::model::geometry::PivotPoint;
use wc3::model::materials::{Texture, TextureFlags};
use wc3::model::mdl;
use wc3::model::mdl::{
    Lexer, Parser, ReadError, ReadErrorKind, Span, TokenKind, WriteError, Writer,
};
use wc3::model::mdl::{Read as _, Write as _};
use wc3::model::mdx::Read as _;
use wc3::model::mdx::Write as _;
use wc3::model::FixedText;
use wc3::model::IoError;

fn print<T: mdl::Write>(value: &T) -> String {
    value.encode_mdl().unwrap()
}

fn print_result(value: &str) -> Result<(), IoError<WriteError>> {
    Writer::new(io::sink()).write(value)
}

mod lexer;
mod parser;
mod records;
mod writer;
