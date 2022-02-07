use cqlparser::parse;

fn main() {
    parse("select field from table");
}
