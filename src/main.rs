mod ast;
mod gen;
mod lex;
mod myria_settings;
mod obj;
mod parse;
mod stdlib;
mod vtable;

use std::{
    env,
    io::{self, Write},
};

use {
    ast::Scope, gen::{run_myria, run_myria_with_scope}, lex::Lexer, myria_settings::MyriaConfig, obj::Primitive,
    parse::Parser,
};

fn main() {
    let config = MyriaConfig::from_args(env::args());
    let Some(program) = config.program_string() else {
        repl(Scope::default());
        return;
    };
    dbg!("{}\n", &program);
    config.load_libs_to_rsc();

    match run_myria(&program) {
        Ok(result) => println!("Result: {result}"),
        Err(err) => gen::print_error(err),
    }
}

fn repl(mut scope: Scope) {
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