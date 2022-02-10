use nom::bytes::complete::tag_no_case;
use nom::sequence::tuple;
use nom::{AsBytes, IResult};

use crate::ast::*;
use crate::nom_bytes::NomBytes;

pub(crate) fn insert(i: NomBytes) -> IResult<NomBytes, Insert> {
    let (remaining_input, _) = tuple((tag_no_case(b"insert".as_bytes()),))(i)?;
    Ok((remaining_input, Insert {}))
}
