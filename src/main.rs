mod ast;
mod gen;
mod lex;
mod parse;
mod stdlib;
mod vtable;

use std::{env, fs, io::Write};

use crate::{
    ast::Primitive,
    lex::{Meta, Token, TokenType},
};

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

    // let tokens = lex::tokenize(&program);

    let tokens = vec![
        Token {
            info: TokenType::Keyword(gen::Keyword::Let),
            metadata: Meta { line_number: 1 },
        },
        Token {
            info: TokenType::Id(String::from("x")),
            metadata: Meta { line_number: 1 },
        },
        Token {
            info: TokenType::Operator(gen::Operator::Assign),
            metadata: Meta { line_number: 1 },
        },
        Token {
            info: TokenType::Id(String::from("import")),
            metadata: Meta { line_number: 1 },
        },
        Token {
            info: TokenType::Operator(gen::Operator::LParen),
            metadata: Meta { line_number: 1 },
        },
        Token {
            info: TokenType::Str(String::from("test.mylang")),
            metadata: Meta { line_number: 1 },
        },
        Token {
            info: TokenType::Operator(gen::Operator::RParen),
            metadata: Meta { line_number: 1 },
        },
        Token {
            info: TokenType::Operator(gen::Operator::Semicolon),
            metadata: Meta { line_number: 1 },
        },

        Token {
            info: TokenType::Id(String::from("print")),
            metadata: Meta { line_number: 2 },
        },
        Token {
            info: TokenType::Operator(gen::Operator::LParen),
            metadata: Meta { line_number: 2 },
        },
        Token {
            info: TokenType::Id(String::from("x")),
            metadata: Meta { line_number: 2 },
        },
        Token {
            info: TokenType::Operator(gen::Operator::RParen),
            metadata: Meta { line_number: 2 },
        },
    ];

    dbg!(&tokens);

    let parser = parse::Parser::new(&tokens);
    let ex = parser.parse();
    dbg!(&ex);

    let result = ex.resolve();
    println!("Result: {result:?}");
}

fn repl() {
    use crate::ast::Scope;
    use crate::lex;
    use crate::parse::Parser;
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

        let toks = lex::tokenize(resp);
        dbg!(&toks);
        let parser = Parser::new(&toks);
        let expr = parser.parse();
        dbg!(&expr);
        let result = expr.evaluate(&mut scope);
        // if result.primitive != Primitive::Null {
        if result.primitive != Primitive::Null {
            println!("{}", result.to_string());
        }
        // println!("{result:?}");
        // }
    }
}
