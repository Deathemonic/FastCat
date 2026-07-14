use criterion::{Criterion, criterion_group, criterion_main};
use fastcat::fconcat;
use std::hint::black_box;
use string_concat::string_concat_impl;

fn impure(buf: &mut String) -> &str {
    for i in 0..10 {
        buf.push_str(&i.to_string());
    }
    buf.as_str()
}

fn bench_mixed(c: &mut Criterion) {
    const CONST: &str = "const ";
    let var = "var ";
    let mut buf = String::new();

    let mut group = c.benchmark_group("mixed");

    group.bench_function("fastcat", |b| {
        b.iter(|| {
            buf.clear();
            black_box(fconcat!(
                "lit0 ",
                const { CONST },
                var,
                "lit1 ",
                "lit2 ",
                impure(&mut buf)
            ))
        });
    });

    group.bench_function("fast-concat", |b| {
        b.iter(|| {
            buf.clear();
            black_box(fast_concat::fast_concat!(
                "lit0 ",
                const CONST,
                var,
                "lit1 ",
                "lit2 ",
                impure(&mut buf)
            ))
        });
    });

    group.bench_function("string-concat", |b| {
        b.iter(|| {
            buf.clear();
            black_box(string_concat::string_concat!(
                "lit0 ",
                CONST,
                var,
                "lit1 ",
                "lit2 ",
                impure(&mut buf)
            ))
        });
    });

    group.bench_function("format!", |b| {
        b.iter(|| {
            buf.clear();
            black_box(format!("lit0 {CONST}{var}lit1 lit2 {}", impure(&mut buf)))
        });
    });

    group.finish();
}

fn bench_all_const(c: &mut Criterion) {
    const A: &str = "hello ";
    const B: &str = "const ";
    const D: &str = "world";

    let mut group = c.benchmark_group("all_const");

    group.bench_function("fastcat", |b| {
        b.iter(|| black_box(fconcat!(const { A }, const { B }, "literal ", const { D })));
    });

    group.bench_function("fast-concat", |b| {
        b.iter(|| black_box(fast_concat::fast_concat!(const A, const B, "literal ", const D)));
    });

    group.bench_function("constcat-direct", |b| {
        b.iter(|| black_box(constcat::concat!(A, B, "literal ", D)));
    });

    group.bench_function("format!", |b| {
        b.iter(|| black_box(format!("{A}{B}literal {D}")));
    });

    group.finish();
}

fn bench_single_dynamic(c: &mut Criterion) {
    let s = String::from("a single dynamic string, unchanged");

    let mut group = c.benchmark_group("single_dynamic");

    group.bench_function("fastcat", |b| {
        b.iter(|| black_box(fconcat!(s.as_str())));
    });

    group.bench_function("fast-concat", |b| {
        b.iter(|| black_box(fast_concat::fast_concat!(s.as_str())));
    });

    group.bench_function("format!", |b| {
        b.iter(|| black_box(s.to_string()));
    });

    group.bench_function("baseline-passthrough", |b| {
        b.iter(|| black_box(s.as_str()));
    });

    group.finish();
}

fn bench_many_small_pieces(c: &mut Criterion) {
    let words: Vec<String> = (0..32).map(|i| format!("w{i}")).collect();
    let refs: Vec<&str> = words.iter().map(String::as_str).collect();

    let mut group = c.benchmark_group("many_small_pieces");

    group.bench_function("fastcat_32", |b| {
        b.iter(|| {
            black_box(fconcat!(
                refs[0], refs[1], refs[2], refs[3], refs[4], refs[5], refs[6], refs[7], refs[8],
                refs[9], refs[10], refs[11], refs[12], refs[13], refs[14], refs[15], refs[16],
                refs[17], refs[18], refs[19], refs[20], refs[21], refs[22], refs[23], refs[24],
                refs[25], refs[26], refs[27], refs[28], refs[29], refs[30], refs[31]
            ))
        });
    });

    group.bench_function("join_32", |b| {
        b.iter(|| black_box(refs.concat()));
    });

    group.bench_function("format!_32", |b| {
        b.iter(|| {
            black_box(format!(
                "{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}{}",
                refs[0],
                refs[1],
                refs[2],
                refs[3],
                refs[4],
                refs[5],
                refs[6],
                refs[7],
                refs[8],
                refs[9],
                refs[10],
                refs[11],
                refs[12],
                refs[13],
                refs[14],
                refs[15],
                refs[16],
                refs[17],
                refs[18],
                refs[19],
                refs[20],
                refs[21],
                refs[22],
                refs[23],
                refs[24],
                refs[25],
                refs[26],
                refs[27],
                refs[28],
                refs[29],
                refs[30],
                refs[31]
            ))
        });
    });

    group.finish();
}

fn bench_sep_mixed(c: &mut Criterion) {
    const CONST: &str = "const ";
    let var = "var ";
    let mut buf = String::new();

    let mut group = c.benchmark_group("sep_mixed");

    group.bench_function("fastcat_sep", |b| {
        b.iter(|| {
            buf.clear();
            black_box(fconcat!(
                "-";
                "lit0", const { CONST }, var, "lit1", "lit2", impure(&mut buf)
            ))
        });
    });

    group.bench_function("format!", |b| {
        b.iter(|| {
            buf.clear();
            black_box(format!("lit0-{CONST}-{var}-lit1-lit2-{}", impure(&mut buf)))
        });
    });

    group.bench_function("manual_push_str", |b| {
        b.iter(|| {
            buf.clear();
            let dynamic = impure(&mut buf).to_string();
            let pieces = ["lit0", CONST, var, "lit1", "lit2", dynamic.as_str()];
            let mut out = String::with_capacity(
                pieces.iter().map(|p| p.len()).sum::<usize>() + (pieces.len() - 1),
            );
            for (i, p) in pieces.iter().enumerate() {
                if i > 0 {
                    out.push('-');
                }
                out.push_str(p);
            }
            black_box(out)
        });
    });

    group.finish();
}

fn bench_sep_all_const(c: &mut Criterion) {
    const A: &str = "hello";
    const B: &str = "const";
    const D: &str = "world";

    let mut group = c.benchmark_group("sep_all_const");

    group.bench_function("fastcat_sep", |b| {
        b.iter(|| black_box(fconcat!("-"; const { A }, const { B }, "literal", const { D })));
    });

    group.bench_function("format!", |b| {
        b.iter(|| black_box(format!("{A}-{B}-literal-{D}")));
    });

    group.finish();
}

fn bench_sep_dynamic_pieces(c: &mut Criterion) {
    let words: Vec<String> = (0..8).map(|i| format!("w{i}")).collect();
    let refs: Vec<&str> = words.iter().map(String::as_str).collect();

    let mut group = c.benchmark_group("sep_dynamic_pieces");

    group.bench_function("fastcat_sep_8", |b| {
        b.iter(|| {
            black_box(fconcat!(
                ", ";
                refs[0], refs[1], refs[2], refs[3], refs[4], refs[5], refs[6], refs[7]
            ))
        });
    });

    group.bench_function("join_8", |b| {
        b.iter(|| black_box(refs.join(", ")));
    });

    group.bench_function("format!_8", |b| {
        b.iter(|| {
            black_box(format!(
                "{}, {}, {}, {}, {}, {}, {}, {}",
                refs[0], refs[1], refs[2], refs[3], refs[4], refs[5], refs[6], refs[7]
            ))
        });
    });

    group.bench_function("manual_push_str_8", |b| {
        b.iter(|| {
            let mut out = String::with_capacity(
                refs.iter().map(|r| r.len()).sum::<usize>() + (refs.len() - 1) * 2,
            );
            for (i, r) in refs.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                out.push_str(r);
            }
            black_box(out)
        });
    });

    group.finish();
}

fn bench_sep_dynamic_separator(c: &mut Criterion) {
    let sep = String::from(" | ");
    let words: Vec<String> = (0..8).map(|i| format!("w{i}")).collect();
    let refs: Vec<&str> = words.iter().map(String::as_str).collect();

    let mut group = c.benchmark_group("sep_dynamic_separator");

    group.bench_function("fastcat_sep_dynamic", |b| {
        b.iter(|| {
            black_box(fconcat!(
                sep.as_str();
                refs[0], refs[1], refs[2], refs[3], refs[4], refs[5], refs[6], refs[7]
            ))
        });
    });

    group.bench_function("join_dynamic", |b| {
        b.iter(|| black_box(refs.join(sep.as_str())));
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_mixed,
    bench_all_const,
    bench_single_dynamic,
    bench_many_small_pieces,
    bench_sep_mixed,
    bench_sep_all_const,
    bench_sep_dynamic_pieces,
    bench_sep_dynamic_separator,
);
criterion_main!(benches);
