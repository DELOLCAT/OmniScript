mod checker;
mod parser;
mod plt;
mod tokenizer;
use std::{path::Path, rc::Rc, time::Instant};
mod types;
mod parser_shared;
use parser::Parser;

use crate::types::Diagnostic;
fn display_diag(d: Diagnostic) {
    println!("{}", d.kind);
    match (d.span.endln, d.span.endcol) {
        (Some(endln), Some(endcol)) => {
            println!(
                "at {}:{}:{} to {}:{}",
                d.span.fp,
                d.span.ln + 1,
                d.span.col + 1,
                endln + 1,
                endcol + 1
            )
        }
        (None, None) => {
            println!("at {}:{}:{}", d.span.fp, d.span.ln + 1, d.span.col + 1)
        }
        _ => unreachable!(),
    };
    for info in &d.info {
        match &info.span {
            Some(span) => match (span.endln, span.endcol) {
                (Some(endln), Some(endcol)) => {
                    println!(
                        "at {}:{}:{} to {}:{} -> {}: {}",
                        span.fp,
                        span.ln + 1,
                        span.col + 1,
                        endln + 1,
                        endcol + 1,
                        info.itype,
                        info.msg
                    )
                }
                (None, None) => {
                    println!(
                        "at {}:{}:{} -> {}: {}",
                        span.fp,
                        span.ln + 1,
                        span.col + 1,
                        info.itype,
                        info.msg
                    )
                }
                _ => unreachable!(),
            },
            None => println!("-> {}: {}", info.itype, info.msg),
        }
    }
}
fn main() {
    let stdlib_path = Path::new("/home/ahmad/kn/");
    let prelude_path = stdlib_path.join("prelude.kn");
    let start = Instant::now();
    let a = tokenizer::Tokenizer::new(
        "\
enum Test {
    Variant {
        foo: {
            bar: str
        }
    }
}

",
        Rc::from("foo.test"),
    );
    let b: Vec<Result<tokenizer::Token, types::Diagnostic>> = a.collect();
    for item in b
        .iter()
        .filter(|x| x.is_err())
        .map(|x| x.as_ref().unwrap_err())
    {
        println!("{}", item.kind);
        println!(
            "at {}:{}:{}",
            item.span.fp,
            item.span.ln + 1,
            item.span.col + 1
        );
        for info in &item.info {
            match &info.span {
                Some(v) => println!(
                    "{}:{}:{} -> {}: {}",
                    v.fp,
                    v.ln + 1,
                    v.col + 1,
                    info.itype,
                    info.msg
                ),
                None => println!("{}: {}", info.itype, info.msg),
            }
        }
        println!("---")
    }
    if b.iter().all(|x| x.is_ok()) {
        println!("{:#?}", b);
        let v = b.iter().map(|x| x.clone().unwrap()).collect();
        let psr = Parser::new(v, Rc::from("test"), prelude_path);

        for item in psr {
            //let item = psr.parse_func();
            // println!("----------");
            // println!("----------");
            // println!("----------");
            // println!("--- parser");
            // println!("----------");
            // println!("----------");
            // println!("----------");
            match item {
                Ok(v) => println!("{:#?}", v),
                Err(item) => {
                    display_diag(item);
                    println!("---")
                }
            }

            // println!("{:#?}", psr.parse_plt_file_stmt().unwrap());
            // println!("{:#?}", psr.parse_plt_file_stmt().unwrap());
            // println!("{:#?}", psr.parse_plt_file_stmt().unwrap());
            // println!("{:#?}", psr.parse_plt_file_stmt().unwrap());
            // println!("{:#?}", psr.parse_plt_file_stmt().unwrap());
            // println!("{:#?}", psr.parse_plt_file_stmt().unwrap());
        }
        let elapsed = start.elapsed();
        println!("{:.2?}", elapsed)
    }
}
