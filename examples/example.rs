use bytes::Bytes;
use cqlparser::parse;

fn main() {
    parse(Bytes::from_static("select field from table".as_bytes()));
}
