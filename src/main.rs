mod ast;
mod gen;
mod lex;
mod obj;
mod parse;
mod stdlib;
mod vtable;

use obj::Primitive;
use std::{env, fs, io::Write};

fn main() {
    let mut args = env::args();
    let _this_path = args.next().expect("Always contains it's own file path");
    let Some(file) = args.next() else {
        repl();
        return;
    };
    let program = fs::read_to_string(&file);
    let Ok(program) = program else {
        panic!("File({}) could not be read: {}", file, program.unwrap_err());
    };
    dbg!("{}\n", &program);

    let tokens = lex::Lexer::new(&program).tokenize();
    dbg!(&tokens);

    let parser = parse::Parser::new(&tokens);
    let ex = parser.parse();
    dbg!(&ex);

    let result = ex.resolve();
    println!("Result: {result:?}");
}

fn repl() {
    use crate::{ast::Scope, lex, parse::Parser};
    let mut scope = Scope::default();

    loop {
        print!("> ");
        std::io::stdout().flush().unwrap();

        let resp = {
            let mut s = String::new();
            std::io::stdin().read_line(&mut s).unwrap();
            s = s.trim().to_string();

            while s.ends_with('\\') {
                s.pop();
                let mut s2 = String::new();
                std::io::stdin().read_line(&mut s2).unwrap();
                s2 = s2.trim().to_string();
                s = format!("{s}\n{s2}");
            }
            s
        };
        let resp = resp.trim();

        if resp == "exit" || resp == "quit" {
            break;
        }

        let lexer = lex::Lexer::new(resp);
        let toks = lexer.tokenize();
        dbg!(&toks);
        let parser = Parser::new(&toks);
        let expr = parser.parse();
        dbg!(&expr);
        let result = expr.evaluate(&mut scope);
        // if result.primitive != Primitive::Null {
        if result.primitive != Primitive::Null {
            println!("{result}");
        }
        // println!("{result:?}");
        // }
    }
}

fn test() -> ! {
    use lex::Lexer;
    use parse::Parser;
    let program = format!("{}\n{}", "var a = empty()", "var a.x = true");
    let lex = Lexer::new(&program);
    let tokens = lex.tokenize();
    let parser = Parser::new(&tokens);
    let expr = parser.parse();

    expr.resolve();
    std::process::exit(0);
}