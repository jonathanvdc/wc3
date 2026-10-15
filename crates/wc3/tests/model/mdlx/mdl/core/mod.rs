use std::io::{self, Write};
use wc3::model::formats::mdlx::animation::{GlobalSequence, Sequence, SequenceFlags};
use wc3::model::formats::mdlx::geometry::PivotPoint;
use wc3::model::formats::mdlx::materials::{Texture, TextureFlags};
use wc3::model::formats::mdlx::mdl;
use wc3::model::formats::mdlx::mdl::{
    Lexer, Parser, ReadError, ReadErrorKind, Span, TokenKind, WriteError, Writer,
};
use wc3::model::formats::mdlx::mdl::{Read as _, Write as _};
use wc3::model::formats::mdlx::mdx::Read as _;
use wc3::model::formats::mdlx::mdx::Write as _;
use wc3::model::formats::mdlx::FixedText;
use wc3::model::formats::mdlx::IoError;

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
