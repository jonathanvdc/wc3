use std::io::{self, Write};
use wc3::model::mdlx::animation::{GlobalSequence, Sequence, SequenceFlags};
use wc3::model::mdlx::geometry::PivotPoint;
use wc3::model::mdlx::materials::{Texture, TextureFlags};
use wc3::model::mdlx::mdl;
use wc3::model::mdlx::mdl::{
    Lexer, Parser, ReadError, ReadErrorKind, Span, TokenKind, WriteError, Writer,
};
use wc3::model::mdlx::mdl::{Read as _, Write as _};
use wc3::model::mdlx::mdx::Read as _;
use wc3::model::mdlx::mdx::Write as _;
use wc3::model::mdlx::FixedText;
use wc3::model::mdlx::IoError;

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
