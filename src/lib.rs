pub mod ast;
pub(crate) mod parser;

use std::str;

use nom::branch::alt;
use nom::combinator::map;
use nom::IResult;

use crate::ast::*;
use crate::parser::insert::insert;
use crate::parser::select::select;

pub fn parse(value: &str) -> Vec<Statement> {
    vec![sql_query(value.as_bytes()).unwrap().1]
}

pub fn sql_query(i: &[u8]) -> IResult<&[u8], Statement> {
    alt((
        map(select, Statement::Select),
        map(insert, Statement::Insert),
    ))(i)
}

// TODO: ooooooh because of all the backtracking we have to do, returning a String would be a performance nightmare.
// Bytes is obviously a lot more efficient to clone than String but maybe we will see minor problems for even Byte due to reference counting?
// We would have to implement and measure.
