use bytes::Bytes;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

use cqlparser::parse;

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("select");

    group.bench_function("simple", |b| {
        b.iter(|| {
            parse(black_box(Bytes::from_static(
                "SELECT * FROM FOO".as_bytes(),
            )))
        })
    });

    {
        let mut lots_of_field = "SELECT field as alias".to_string();
        for _ in 0..100 {
            lots_of_field.push_str(", field as alias")
        }
        lots_of_field.push_str(" FROM table");
        let lots_of_field = Bytes::from(lots_of_field.as_bytes().to_vec());
        group.bench_function("lots_of_field", |b| {
            b.iter(|| parse(black_box(lots_of_field.clone())))
        });
    }

    {
        let mut lots_of_where = "SELECT field FROM table WHERE foo = 'some string'".to_string();
        for _ in 0..100 {
            lots_of_where.push_str(" AND foo = 'some string'")
        }
        let lots_of_where = Bytes::from(lots_of_where.as_bytes().to_vec());
        group.bench_function("lots_of_where", |b| {
            b.iter(|| parse(black_box(Bytes::from(lots_of_where.clone()))))
        });
    }

    group.bench_function("christmas_tree", |b| {
        b.iter(|| parse(black_box(Bytes::from_static("SELECT distinct json field1, field2 as foo FROM table order by order_column WHERE foo = 1 DESC limit 9999 allow filtering".as_bytes()))))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
