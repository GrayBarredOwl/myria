use crate::ast::{Expression, Object};
use crate::gen::{self, keywords, operators, Operator};

#[derive(PartialEq, Debug, Clone)]
pub enum TokenType {
    Int(i64),
    Float(f64),
    Char(char),
    Str(String),
    Id(String),
    Keyword(gen::Keyword),
    Operator(gen::Operator),
}

impl TokenType {
    pub fn is_value(&self) -> bool {
        matches!(
            self,
            Self::Int(_) | Self::Float(_) | Self::Str(_) | Self::Id(_)
        )
    }
    pub fn is_unary_op(&self) -> bool {
        matches!(
            self,
            Self::Operator(Operator::Minus) | Self::Operator(Operator::Not)
        )
    }

    pub fn is_binary_op(&self) -> bool {
        type Op = Operator;
        let o = match self {
            Self::Operator(Operator::Not) => return false,
            Self::Operator(o) => o,
            _ => return false,
        };

        matches!(
            *o,
            Op::And
                | Op::Assign
                | Op::Dot
                | Op::Equals
                | Op::Gr
                | Op::GrEq
                | Op::Lt
                | Op::LtEq
                | Op::Minus
                | Op::NEqual
                | Op::Or
                | Op::Plus
                | Op::Slash
                | Op::Star
                | Op::PlusEq
                | Op::MinusEq
                | Op::StarEq
                | Op::SlashEq
        )
    }
    pub fn is_semi(&self) -> bool {
        matches!(self, TokenType::Operator(Operator::Semicolon))
    }

    pub fn to_value(&self) -> Option<Expression> {
        if self.is_value() {
            Some(match self {
                Self::Int(n) => Object::make_int(*n).to_expr(),
                Self::Float(n) => Object::make_float(*n).to_expr(),
                Self::Char(c) => Object::make_char(*c).to_expr(),
                Self::Id(name) => Expression::Variable(name.clone()),
                _ => panic!(),
            })
        } else {
            None
        }
    }
    pub fn to_op(&self) -> Option<Operator> {
        match self {
            Self::Operator(o) => Some(*o),
            _ => None,
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Meta {
    pub line_number: usize,
}

#[derive(Clone)]
pub struct Token {
    pub info: TokenType,
    pub metadata: Meta,
}
impl Default for Token {
    fn default() -> Self {
        Self {
            info: TokenType::Operator(Operator::Semicolon),
            metadata: Meta { line_number: 0 },
        }
    }
}

impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Token({:?})", self.info)
    }
}

impl Token {
    fn new(ttype: TokenType, tmeta: Meta) -> Self {
        Token {
            info: ttype,
            metadata: tmeta,
        }
    }
    fn is_semi(&self) -> bool {
        match self.info {
            TokenType::Operator(gen::Operator::Semicolon) => true,
            _ => false,
        }
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.info == other.info
    }
}

pub fn tokenize(program: &str) -> Vec<Token> {
    let mut vec = vec![];
    for (line_number, line) in program.lines().enumerate() {
        let line_number = line_number + 1; // enumeratate starts at 0, not 1

        let line = line.split("//").next().expect("Split len > 0"); // Remove everything after //
        let toks = line.split_whitespace().map(|s| {
            let t = resolve_token(s, line_number);
            if t.is_err() {
                eprintln!("{}", t.unwrap_err());
                std::process::exit(1);
            }
            t.unwrap()
        });
        vec.extend(toks);
        if !vec.is_empty() && !vec.last().unwrap().is_semi() {
            vec.push(Token::new(
                TokenType::Operator(gen::Operator::Semicolon),
                Meta { line_number },
            ));
        }
    }
    remove_initial_and_trailing_semis(&mut vec);
    vec
}

pub fn tokenize2(program: &str) -> Vec<Token> {
    let mut vec = vec![];
    let mut i = 0;
    let mut cur_line = 1;
    while i < program.len() {
        let c = program.chars().nth(i).unwrap();
        if c.is_whitespace() {
            cur_line += (c == '\n') as usize;
            i += 1;
            continue;
        } else if let Some(op) = operators().get(&program[i..=i]) {
            vec.push(Token::new(
                TokenType::Operator(*op),
                Meta {
                    line_number: cur_line,
                },
            ));
        } else if c.is_numeric() {
            let start = i;
            while program.chars().nth(i).unwrap().is_numeric() {
                i += 1;
            }
            let next = program.chars().nth(i).unwrap();
            if next.is_whitespace()
                || operators()
                    .keys()
                    .any(|k| k.chars().nth(0).unwrap() == next)
            {
                vec.push(Token::new(
                    TokenType::Int(program[start..i].parse().unwrap()),
                    Meta {
                        line_number: cur_line,
                    },
                ));
            } else if next == '.' {
                while program.chars().nth(i).unwrap().is_numeric() {
                    i += 1;
                }
                vec.push(Token::new(
                    TokenType::Float(program[start..i].parse().unwrap()),
                    Meta {
                        line_number: cur_line,
                    },
                ));
            }
            i -= 1;
        } else if c.is_alphabetic() || c == '_' {
            let start = i;
            while program.chars().nth(i).unwrap().is_alphanumeric()
                || program.chars().nth(i).unwrap() == '_'
            {
                i += 1;
            }
            let token = &program[start..i];
            if keywords().contains_key(token) {
                vec.push(Token::new(
                    TokenType::Keyword(*keywords().get(token).unwrap()),
                    Meta {
                        line_number: cur_line,
                    },
                ));
            } else {
                vec.push(Token::new(
                    TokenType::Id(token.to_string()),
                    Meta {
                        line_number: cur_line,
                    },
                ));
            }
            i -= 1;
        }

        i += 1;
    }

    vec
}
pub fn tokenize3(tokens: &str) -> Vec<Token> {
    let vec = vec![];
    
    

    vec
}

fn remove_initial_and_trailing_semis(tokens: &mut Vec<Token>) {
    while tokens.len() > 0 && tokens.first().unwrap().is_semi() {
        tokens.remove(0);
    }
    while tokens.len() > 0 && tokens[tokens.len() - 2].is_semi() {
        tokens.pop();
    }
}

fn resolve_token(token: &str, line_number: usize) -> Result<Token, String> {
    if token.chars().count() == 0 {
        return Err(String::from("Token must contain > 0 characters"));
    }
    type TT = TokenType;

    let meta = Meta { line_number };

    if let Some(op) = gen::operators().get(&token) {
        Ok(Token::new(TT::Operator(*op), meta))
    } else if let Some(kw) = gen::keywords().get(&token) {
        Ok(Token::new(TT::Keyword(*kw), meta))
    } else if token.chars().next().expect("Checked len > 0").is_numeric() {
        resolve_number(token, line_number)
    } else {
        // Default is an identifier
        resolve_id(token, line_number)
        // Err(format!("Unrecognized token: '{token}' on line {line_number}"))
    }
}

fn resolve_number(num: &str, line_number: usize) -> Result<Token, String> {
    if num.chars().count() == 0 {
        return Err(format!("(line: {line_number}) Number can not be empty"));
    }

    let meta = Meta { line_number };
    let mut is_int = true;

    for c in num.chars() {
        if c == '.' {
            if !is_int {
                return Err(format!(
                    "(line: {line_number}) Number can not contain multiple dots"
                ));
            }
            is_int = false;
        } else if !c.is_numeric() {
            return Err(format!("(line: {line_number}) Not a number: {num}"));
        }
    }

    let tt = match is_int {
        true => TokenType::Int(num.parse::<i64>().expect("Checked it's valid")),
        false => TokenType::Float(num.parse::<f64>().expect("Checked it's valid")),
    };

    Ok(Token::new(tt, meta))
}

fn resolve_id(id: &str, line_number: usize) -> Result<Token, String> {
    if id.chars().count() == 0 {
        return Err(String::from("Identifer can not be empty"));
    }

    let meta = Meta { line_number };

    let mut it = id.chars();
    let mut c = it.next();

    let ch = c.expect("len > 0");
    if !ch.is_alphabetic() || ch == '_' {
        return Err(format!("(line: {line_number}) Invalid identifier: {id} <- must start with alphabetical character or underscore"));
    }

    c = it.next();
    while c.is_some() {
        let ch = c.expect("Checked .is_some()");

        if !(ch.is_alphanumeric() || ch == '_') {
            return Err(format!("(line: {line_number}) Invalid identifier: {id} <- can only contain alphanumeric characters and underscores"));
        }

        c = it.next();
    }

    Ok(Token::new(TokenType::Id(id.to_string()), meta))
}
