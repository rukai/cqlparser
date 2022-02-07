pub mod ast;

use ast::*;
use nom::branch::alt;
use nom::bytes::complete::{tag, tag_no_case, take_while1};
use nom::character::complete::{digit1, multispace0, multispace1};
use nom::character::is_alphanumeric;
use nom::combinator::{map, opt};
use nom::multi::many0;
use nom::sequence::{delimited, preceded, terminated, tuple};
use nom::IResult;
use std::str;
use std::str::FromStr;

pub fn parse(value: &str) -> Vec<Statement> {
    vec![sql_query(value.as_bytes()).unwrap().1]
}

pub fn sql_query(i: &[u8]) -> IResult<&[u8], Statement> {
    alt((
        map(select, Statement::Select),
        map(select, Statement::Select),
    ))(i)
}

pub fn select(i: &[u8]) -> IResult<&[u8], Select> {
    let (remaining_input, (_, _, json, select, from, order_by, limit, allow_filtering)) = tuple((
        tag_no_case("select"),
        multispace1,
        json,
        select_fields,
        select_from,
        opt(order_by),
        limit,
        allow_filtering,
    ))(i)?;
    Ok((
        remaining_input,
        Select {
            distinct: false,
            json,
            select,
            from,
            where_: vec![],
            order_by,
            limit,
            allow_filtering,
        },
    ))
}

pub fn json(i: &[u8]) -> IResult<&[u8], bool> {
    opt(terminated(tag_no_case("json"), multispace1))(i).map(|(r, v)| (r, v.is_some()))
}

pub fn order_by(i: &[u8]) -> IResult<&[u8], OrderBy> {
    let (remaining_input, (_, _, _, _, _, name, ordering)) = tuple((
        multispace1,
        tag_no_case("order"),
        multispace1,
        tag_no_case("by"),
        multispace1,
        identifier,
        opt(preceded(multispace1, ordering)),
    ))(i)?;

    let name = String::from_utf8(name.to_vec()).unwrap();
    let ordering = ordering.unwrap_or(Ordering::Asc);
    Ok((remaining_input, OrderBy { name, ordering }))
}

pub fn ordering(i: &[u8]) -> IResult<&[u8], Ordering> {
    alt((
        map(tag_no_case("asc"), |_| Ordering::Asc),
        map(tag_no_case("desc"), |_| Ordering::Desc),
    ))(i)
}

pub fn limit(i: &[u8]) -> IResult<&[u8], Option<u64>> {
    opt(preceded(
        tuple((multispace1, tag_no_case("limit"), multispace1)),
        unsigned_number,
    ))(i)
}

pub fn unsigned_number(i: &[u8]) -> IResult<&[u8], u64> {
    map(digit1, |d| {
        FromStr::from_str(str::from_utf8(d).unwrap()).unwrap()
    })(i)
}

pub fn allow_filtering(i: &[u8]) -> IResult<&[u8], bool> {
    opt(preceded(multispace1, tag_no_case("allow filtering")))(i).map(|(r, v)| (r, v.is_some()))
}

pub fn select_fields(i: &[u8]) -> IResult<&[u8], Vec<SelectElement>> {
    many0(terminated(
        alt((
            map(tag("*"), |_| SelectElement {
                expr: Expr::Wildcard,
                as_alias: None,
            }),
            map(identifier, |name| SelectElement {
                expr: Expr::Name(String::from_utf8(name.to_vec()).unwrap()),
                as_alias: None,
            }),
        )),
        opt(ws_sep_comma),
    ))(i)
}

pub fn select_from(i: &[u8]) -> IResult<&[u8], Vec<String>> {
    preceded(
        tuple((multispace1, tag_no_case("from"), multispace1)),
        map(identifier, |name| {
            vec![String::from_utf8(name.to_vec()).unwrap()]
        }),
    )(i)
}

pub(crate) fn ws_sep_comma(i: &[u8]) -> IResult<&[u8], &[u8]> {
    delimited(multispace0, tag(","), multispace0)(i)
}

pub fn identifier(i: &[u8]) -> IResult<&[u8], &[u8]> {
    take_while1(is_identifier)(i)
}

pub fn is_identifier(chr: u8) -> bool {
    is_alphanumeric(chr) || chr == b'_'
}

// TODO: ooooooh because of all the backtracking we have to do, returning a String would be a performance nightmare.
// Bytes is obviously a lot more efficient to clone than String but maybe we will see minor problems for even Byte due to reference counting?
// We would have to implement and measure.
