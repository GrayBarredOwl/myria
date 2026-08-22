use std::{collections::HashMap, sync::OnceLock};

pub const EXTENSION: &str = ".myria";

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Keyword {
    Let,
    Var,
    Func,
    Class,

    If,
    Elif,
    Else,
    Loop,
    Call,
    Return,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Operator {
    Plus,
    Minus,
    Star,
    Slash,

    Assign,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,

    Equals,
    NEqual,
    Gr,
    Lt,
    GrEq,
    LtEq,

    And,
    Or,
    Not,

    Dot,
    Comma,
    Semicolon,
    LParen,
    RParen,
    LCurly,
    RCurly,
    LBracket,
    RBracket,
}
impl Operator {
    pub fn is_opening(self) -> bool {
        matches!(self, Self::LParen | Self::LCurly | Self::LBracket)
    }
    pub fn is_closing(self) -> bool {
        matches!(self, Self::RParen | Self::RBracket | Self::RCurly)
    }
    // pub fn is_grouping(self) -> bool {
    //     self.is_closing() | self.is_opening()
    // }
}

pub fn keywords() -> &'static HashMap<&'static str, Keyword> {
    static KEYWORDS: OnceLock<HashMap<&str, Keyword>> = OnceLock::new();

    KEYWORDS.get_or_init(|| {
        let mut kw = HashMap::new();

        kw.insert("let", Keyword::Let);
        kw.insert("var", Keyword::Var);
        kw.insert("func", Keyword::Func);
        kw.insert("class", Keyword::Class);

        kw.insert("if", Keyword::If);
        kw.insert("elif", Keyword::Elif);
        kw.insert("else", Keyword::Else);
        kw.insert("loop", Keyword::Loop);
        kw.insert("call", Keyword::Call);
        kw.insert("return", Keyword::Return);

        kw
    })
}

pub fn operators() -> &'static HashMap<&'static str, Operator> {
    static OPERATORS: OnceLock<HashMap<&str, Operator>> = OnceLock::new();
    OPERATORS.get_or_init(|| {
        let mut op = HashMap::new();

        op.insert("+", Operator::Plus);
        op.insert("-", Operator::Minus);
        op.insert("*", Operator::Star);
        op.insert("/", Operator::Slash);

        op.insert("=", Operator::Assign);
        op.insert("+=", Operator::PlusEq);
        op.insert("-=", Operator::MinusEq);
        op.insert("*=", Operator::StarEq);
        op.insert("/=", Operator::SlashEq);

        op.insert("==", Operator::Equals);
        op.insert("!=", Operator::NEqual);
        op.insert(">", Operator::Gr);
        op.insert("<", Operator::Lt);
        op.insert(">=", Operator::GrEq);
        op.insert("<=", Operator::LtEq);

        op.insert("&", Operator::And);
        op.insert("|", Operator::Or);
        op.insert("!", Operator::Not);

        op.insert(".", Operator::Dot);
        op.insert(",", Operator::Comma);
        op.insert(";", Operator::Semicolon);
        op.insert("(", Operator::LParen);
        op.insert(")", Operator::RParen);
        op.insert("{", Operator::LCurly);
        op.insert("}", Operator::RCurly);
        op.insert("[", Operator::LBracket);
        op.insert("]", Operator::RBracket);

        op
    })
}

use crate::ast::Scope;
use crate::obj::Object;
use std::io;

pub fn run_file(fp: &str) -> io::Result<Object> {
    run_file_with(fp, &mut Scope::default())
}
pub fn run_file_with(fp: &str, scope: &mut Scope) -> io::Result<Object> {
    use crate::{lex::Lexer, parse::Parser};
    use std::fs;

    let program = fs::read_to_string(fp)?;
    let tokens = Lexer::new(&program).tokenize();

    let expr = Parser::new(&tokens).parse();
    Ok(expr.evaluate(scope))
}
