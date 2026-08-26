mod myria_settings;
mod ast;
mod gen;
mod lex;
mod obj;
mod parse;
mod stdlib;
mod vtable;

use std::{env, io::{self, Write}};

use {
    lex::Lexer,
    parse::Parser,
    obj::{Object, Primitive},
    ast::{Scope, MyriaErr},
    myria_settings::MyriaConfig,
};

fn main() {
    let config = MyriaConfig::from_args(env::args());
    let Some(program) = config.program_string() else {
        repl();
        return;
    };
    dbg!("{}\n", &program);
    match run_myria(&program) {
        Ok(result) => println!("Result: {result}"),
        Err(err) => gen::print_error(err),
    }
}

fn repl() {
    let mut scope = Scope::default();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let resp = {
            let mut s = String::new();
            io::stdin().read_line(&mut s).unwrap();
            s = s.trim().to_string();

            while s.ends_with('\\') {
                s.pop();
                let mut s2 = String::new();
                io::stdin().read_line(&mut s2).unwrap();
                let s2 = s2.trim();
                s = format!("{s}\n{s2}");
            }
            s
        };
        let resp = resp.trim();

        if resp == "exit" || resp == "quit" {
            break;
        };
        match run_myria_with_scope(resp, &mut scope) {
            Ok(result) => {
                if result.primitive != Primitive::Null {
                    println!("{result}");
                    // println!("{result:?}");
                }
            }
            Err(err) => gen::print_error(err),
        }
    }
}

fn run_myria(program: &str) -> Result<Object, MyriaErr> {
    run_myria_with_scope(program, &mut Scope::default())
}
fn run_myria_with_scope(program: &str, scope: &mut Scope) -> Result<Object, MyriaErr> {
    let lexer = Lexer::new(program);
    let toks = lexer.tokenize();
    dbg!(&toks);
    let parser = Parser::new(&toks);
    let expr = parser.parse();
    dbg!(&expr);
    expr.evaluate(scope)
}
