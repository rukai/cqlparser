use std::str;
use std::str::FromStr;

use bytes::Bytes;
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag, tag_no_case, take_while1};
use nom::character::complete::{digit1, multispace0, multispace1};
use nom::character::is_alphanumeric;
use nom::combinator::{map, opt};
use nom::multi::{fold_many0, many0};
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::IResult;

use crate::ast::*;
use crate::nom_bytes::NomBytes;

#[inline(always)]
pub(crate) fn select(i: NomBytes) -> IResult<NomBytes, Select> {
    let (
        remaining_input,
        (_, _, distinct, json, select, from, where_, order_by, limit, allow_filtering),
    ) = tuple((
        tag_no_case("select".as_bytes()),
        multispace1,
        distinct,
        json,
        fields,
        from,
        where_,
        opt(order_by),
        limit,
        allow_filtering,
    ))(i)?;
    Ok((
        remaining_input,
        Select {
            distinct,
            json,
            select,
            from,
            where_,
            order_by,
            limit,
            allow_filtering,
        },
    ))
}

#[inline(always)]
fn json(i: NomBytes) -> IResult<NomBytes, bool> {
    map(
        opt(terminated(tag_no_case("json".as_bytes()), multispace1)),
        |v| v.is_some(),
    )(i)
}

#[inline(always)]
fn distinct(i: NomBytes) -> IResult<NomBytes, bool> {
    map(
        opt(terminated(tag_no_case("distinct".as_bytes()), multispace1)),
        |v| v.is_some(),
    )(i)
}

#[inline(always)]
fn where_(i: NomBytes) -> IResult<NomBytes, Vec<RelationElement>> {
    map(
        opt(preceded(
            tuple((multispace1, tag_no_case("where".as_bytes()), multispace1)),
            where_elements,
        )),
        |x| x.unwrap_or_default(),
    )(i)
}

#[inline(always)]
fn where_elements(i: NomBytes) -> IResult<NomBytes, Vec<RelationElement>> {
    many0(terminated(
        where_element,
        opt(tuple((
            multispace1,
            tag_no_case("AND".as_bytes()),
            multispace1,
        ))), // TODO: this seems wrong
    ))(i)
}

#[inline(always)]
fn where_element(i: NomBytes) -> IResult<NomBytes, RelationElement> {
    let (remaining_input, (lhs, _, operator, _, rhs)) =
        tuple((expr, multispace1, operator, multispace1, expr))(i)?;

    Ok((
        remaining_input,
        RelationElement::Comparison(RelationComparison { lhs, operator, rhs }),
    ))
}

#[inline(always)]
fn order_by(i: NomBytes) -> IResult<NomBytes, OrderBy> {
    let (remaining_input, (_, _, _, _, _, name, ordering)) = tuple((
        multispace1,
        tag_no_case("order".as_bytes()),
        multispace1,
        tag_no_case("by".as_bytes()),
        multispace1,
        identifier,
        opt(preceded(multispace1, ordering)),
    ))(i)?;

    let name = String::from_utf8(name.to_vec()).unwrap();
    let ordering = ordering.unwrap_or(Ordering::Asc);
    Ok((remaining_input, OrderBy { name, ordering }))
}

#[inline(always)]
fn operator(i: NomBytes) -> IResult<NomBytes, ComparisonOperator> {
    alt((
        map(tag("=".as_bytes()), |_| ComparisonOperator::Equals),
        map(tag(">=".as_bytes()), |_| {
            ComparisonOperator::GreaterThanOrEqualTo
        }),
        map(tag(">".as_bytes()), |_| ComparisonOperator::GreaterThan),
        map(tag("<=".as_bytes()), |_| {
            ComparisonOperator::LessThanOrEqualTo
        }),
        map(tag("<".as_bytes()), |_| ComparisonOperator::LessThan),
    ))(i)
}

#[inline(always)]
fn ordering(i: NomBytes) -> IResult<NomBytes, Ordering> {
    alt((
        map(tag_no_case("asc".as_bytes()), |_| Ordering::Asc),
        map(tag_no_case("desc".as_bytes()), |_| Ordering::Desc),
    ))(i)
}

#[inline(always)]
fn limit(i: NomBytes) -> IResult<NomBytes, Option<u64>> {
    opt(preceded(
        tuple((multispace1, tag_no_case("limit".as_bytes()), multispace1)),
        unsigned_number,
    ))(i)
}

#[inline(always)]
fn unsigned_number(i: NomBytes) -> IResult<NomBytes, u64> {
    map(digit1, |d: NomBytes| {
        FromStr::from_str(str::from_utf8(&d).unwrap()).unwrap()
    })(i)
}

#[inline(always)]
fn allow_filtering(i: NomBytes) -> IResult<NomBytes, bool> {
    opt(preceded(
        multispace1,
        tag_no_case("allow filtering".as_bytes()),
    ))(i)
    .map(|(r, v)| (r, v.is_some()))
}

#[inline(always)]
fn fields(i: NomBytes) -> IResult<NomBytes, Vec<SelectElement>> {
    many0(terminated(field, opt(ws_sep_comma)))(i) // TODO: this seems wrong
}

#[inline(always)]
fn field(i: NomBytes) -> IResult<NomBytes, SelectElement> {
    let (remaining, (expr, as_alias)) = pair(
        expr,
        opt(preceded(
            tuple((multispace1, tag_no_case("AS".as_bytes()), multispace1)),
            identifier,
        )),
    )(i)?;

    Ok((remaining, SelectElement { expr, as_alias }))
}

#[inline(always)]
fn from(i: NomBytes) -> IResult<NomBytes, Vec<Bytes>> {
    preceded(
        tuple((multispace1, tag_no_case("from".as_bytes()), multispace1)),
        map(identifier, |name| vec![name]),
    )(i)
}

#[inline(always)]
fn expr(i: NomBytes) -> IResult<NomBytes, Expr> {
    alt((
        map(tag("*".as_bytes()), |_| Expr::Wildcard),
        map(constant, Expr::Constant),
        map(identifier, |name| Expr::Name(name)),
    ))(i)
}

#[inline(always)]
fn constant(i: NomBytes) -> IResult<NomBytes, Constant> {
    alt((
        map(integer_constant, Constant::Decimal),
        map(string_constant, Constant::String),
        map(bool_constant, Constant::Bool),
    ))(i)
}

#[inline(always)]
fn integer_constant(i: NomBytes) -> IResult<NomBytes, i64> {
    map(
        pair(opt(tag("-".as_bytes())), digit1),
        |(negative, bytes): (Option<NomBytes>, NomBytes)| {
            let mut intval = i64::from_str(str::from_utf8(&bytes).unwrap()).unwrap();
            if negative.is_some() {
                intval *= -1;
            }
            intval
        },
    )(i)
}

#[inline(always)]
fn string_constant(i: NomBytes) -> IResult<NomBytes, String> {
    map(raw_string_quoted, |bytes| String::from_utf8(bytes).unwrap())(i)
}

#[inline(always)]
fn raw_string_quoted(i: NomBytes) -> IResult<NomBytes, Vec<u8>> {
    delimited(
        tag("'".as_bytes()),
        fold_many0(
            alt((
                is_not("'".as_bytes()), //
                map(tag("''".as_bytes()), |_: NomBytes| {
                    NomBytes::from(Bytes::from_static(b"'"))
                }),
            )),
            Vec::new,
            |mut acc: Vec<u8>, bytes: NomBytes| {
                acc.extend(bytes.iter());
                acc
            },
        ),
        tag("'".as_bytes()),
    )(i)
}

#[inline(always)]
fn bool_constant(i: NomBytes) -> IResult<NomBytes, bool> {
    alt((
        map(tag_no_case("true".as_bytes()), |_| true),
        map(tag_no_case("false".as_bytes()), |_| false),
    ))(i)
}

#[inline(always)]
pub(crate) fn ws_sep_comma(i: NomBytes) -> IResult<NomBytes, ()> {
    delimited(multispace0, map(tag(",".as_bytes()), |_| ()), multispace0)(i)
}

#[inline(always)]
fn identifier(i: NomBytes) -> IResult<NomBytes, Bytes> {
    map(take_while1(is_identifier), |x: NomBytes| x.into_bytes())(i)
}

#[inline(always)]
fn is_identifier(chr: u8) -> bool {
    is_alphanumeric(chr) || chr == b'_'
}
