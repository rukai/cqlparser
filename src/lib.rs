pub mod ast;
pub(crate) mod nom_bytes;
pub(crate) mod parser;

use bytes::Bytes;
use nom::branch::alt;
use nom::combinator::map;
use nom::IResult;

use crate::ast::*;
use crate::nom_bytes::NomBytes;
use crate::parser::insert::insert;
use crate::parser::select::select;

pub fn parse(value: Bytes) -> Vec<Statement> {
    vec![sql_query(NomBytes::from(value)).unwrap().1]
}

#[inline(always)]
fn sql_query(i: NomBytes) -> IResult<NomBytes, Statement> {
    alt((
        map(select, Statement::Select),
        map(insert, Statement::Insert),
    ))(i)
}

// TODO: ooooooh because of all the backtracking we have to do, returning a String would be a performance nightmare.
// Bytes is obviously a lot more efficient to clone than String but maybe we will see minor problems for even Byte due to reference counting?
// We would have to implement and measure.
