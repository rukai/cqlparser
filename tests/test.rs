use cqlparser::ast::*;
use cqlparser::parse;

fn assert_parses(input: &[&str], ast: Vec<Statement>) {
    for input in input {
        assert_eq!(parse(input), ast);
    }
}

#[test]
fn test_insert() {
    assert_parses(&["insert"], vec![Statement::Insert(Insert {})]);
}

#[test]
fn test_select_one_field() {
    assert_parses(
        &[
            "select field from table",
            "SELECT    field    FROM    table",
        ],
        vec![Statement::Select(Select {
            distinct: false,
            json: false,
            select: vec![SelectElement {
                expr: Expr::Name("field".to_string()),
                as_alias: None,
            }],
            from: vec!["table".to_string()],
            where_: vec![],
            order_by: None,
            limit: None,
            allow_filtering: false,
        })],
    );
}

#[test]
fn test_select_json() {
    assert_parses(
        &[
            "SELECT json field FROM table",
            "SELECT    JSON    field FROM table",
        ],
        vec![Statement::Select(Select {
            distinct: false,
            json: true,
            select: vec![SelectElement {
                expr: Expr::Name("field".to_string()),
                as_alias: None,
            }],
            from: vec!["table".to_string()],
            where_: vec![],
            order_by: None,
            limit: None,
            allow_filtering: false,
        })],
    );
}

#[test]
fn test_select_order_by_asc() {
    assert_parses(
        &[
            "SELECT field FROM table order by pk_field",
            "SELECT field FROM table ORDER BY     pk_field",
            "SELECT field FROM table order   BY    pk_field asc",
            "SELECT field FROM table ORDER     BY     pk_field    ASC",
        ],
        vec![Statement::Select(Select {
            distinct: false,
            json: false,
            select: vec![SelectElement {
                expr: Expr::Name("field".to_string()),
                as_alias: None,
            }],
            from: vec!["table".to_string()],
            where_: vec![],
            order_by: Some(OrderBy {
                name: "pk_field".to_string(),
                ordering: Ordering::Asc,
            }),
            limit: None,
            allow_filtering: false,
        })],
    );
}

#[test]
fn test_select_order_by_desc() {
    assert_parses(
        &[
            "SELECT field FROM table order by foo desc",
            "SELECT field FROM table ORDER     BY     foo    DESC",
        ],
        vec![Statement::Select(Select {
            distinct: false,
            json: false,
            select: vec![SelectElement {
                expr: Expr::Name("field".to_string()),
                as_alias: None,
            }],
            from: vec!["table".to_string()],
            where_: vec![],
            order_by: Some(OrderBy {
                name: "foo".to_string(),
                ordering: Ordering::Desc,
            }),
            limit: None,
            allow_filtering: false,
        })],
    );
}

#[test]
fn test_select_limit_42() {
    assert_parses(
        &[
            "SELECT field FROM table limit 42",
            "SELECT field FROM table    LIMIT    42",
        ],
        vec![Statement::Select(Select {
            distinct: false,
            json: false,
            select: vec![SelectElement {
                expr: Expr::Name("field".to_string()),
                as_alias: None,
            }],
            from: vec!["table".to_string()],
            where_: vec![],
            order_by: None,
            limit: Some(42),
            allow_filtering: false,
        })],
    );
}

#[test]
fn test_select_limit_0() {
    assert_parses(
        &[
            "SELECT field FROM table limit 0",
            "SELECT field FROM table    LIMIT    0",
        ],
        vec![Statement::Select(Select {
            distinct: false,
            json: false,
            select: vec![SelectElement {
                expr: Expr::Name("field".to_string()),
                as_alias: None,
            }],
            from: vec!["table".to_string()],
            where_: vec![],
            order_by: None,
            limit: Some(0),
            allow_filtering: false,
        })],
    );
}

#[test]
fn test_allow_filtering() {
    assert_parses(
        &[
            "SELECT field FROM table allow filtering",
            "SELECT field FROM table    ALLOW FILTERING",
        ],
        vec![Statement::Select(Select {
            distinct: false,
            json: false,
            select: vec![SelectElement {
                expr: Expr::Name("field".to_string()),
                as_alias: None,
            }],
            from: vec!["table".to_string()],
            where_: vec![],
            order_by: None,
            limit: None,
            allow_filtering: true,
        })],
    );
}

#[test]
fn test_select_two_fields() {
    assert_parses(
        &[
            "SELECT field1,field2 FROM table",
            "SELECT field1, field2 FROM table",
            "SELECT   field1  ,   field2    FROM    table",
        ],
        vec![Statement::Select(Select {
            distinct: false,
            json: false,
            select: vec![
                SelectElement {
                    expr: Expr::Name("field1".to_string()),
                    as_alias: None,
                },
                SelectElement {
                    expr: Expr::Name("field2".to_string()),
                    as_alias: None,
                },
            ],
            from: vec!["table".to_string()],
            where_: vec![],
            order_by: None,
            limit: None,
            allow_filtering: false,
        })],
    );
}

#[test]
fn test_select_all() {
    assert_parses(
        &[
            "SELECT * FROM foo",
            "SELECT        *        FROM        foo",
        ],
        vec![Statement::Select(Select {
            distinct: false,
            json: false,
            select: vec![SelectElement {
                expr: Expr::Wildcard,
                as_alias: None,
            }],
            from: vec!["foo".to_string()],
            where_: vec![],
            order_by: None,
            limit: None,
            allow_filtering: false,
        })],
    );
}

#[test]
fn test_select_christmas_tree() {
    assert_parses(
        &["SELECT json field1, field2 FROM table order by order_column DESC limit 9999 allow filtering"],
        vec![Statement::Select(Select {
            distinct: false,
            json: true,
            select: vec![
                SelectElement {
                    expr: Expr::Name("field1".to_string()),
                    as_alias: None,
                },
                SelectElement {
                    expr: Expr::Name("field2".to_string()),
                    as_alias: None,
                },
            ],
            from: vec!["table".to_string()],
            where_: vec![],
            order_by: Some(OrderBy {
                name: "order_column".to_string(),
                ordering: Ordering::Desc,
            }),
            limit: Some(9999),
            allow_filtering: true,
        })],
    );
}
