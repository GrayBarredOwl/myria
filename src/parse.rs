use crate::{
    ast::{BinOpExpr, Expression, IfExpr, LoopExpr, TryExpr, UnOpExpr, VarData},
    lex::{Token, TokenType},
    util::{
        Keyword,
        Operator::{self, RParen},
    },
};

#[derive(Clone, Copy, Debug)]
pub struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, current: 0 }
    }
    pub fn parse(mut self) -> Expression {
        let mut total: Option<Expression> = None;
        while !self.is_finished() {
            let Some(next_expr) = self.parse_expr() else {
                break;
            };

            match total {
                Some(ref mut expr) => expr.push(next_expr),
                None => total = Some(next_expr),
            }
            // dbg!(&total);
        }
        match total {
            Some(Expression::BlockExpr(vec)) => {
                Expression::BlockExpr(vec.into_iter().filter(|e| !e.is_null_lit()).collect())
            }
            Some(expr) => expr,
            None => Expression::default(),
        }
    }
    fn parse_expr(&mut self) -> Option<Expression> {
        type KW = Keyword;
        type TT = TokenType;

        let mut cur_expr = Expression::default();

        if self.is_finished() {
            return None;
        }

        let next_token = self.peek().info.clone();
        if next_token.is_semi() {
            self.consume();
            if let Some(expr) = self.parse_expr() {
                cur_expr = expr;
            }
        } else if matches!(next_token, TT::Keyword(KW::Let) | TT::Keyword(KW::Var)) {
            cur_expr.push(self.parse_var_dec());
            cur_expr.push(self.parse_expr().unwrap());
        } else if matches!(next_token, TT::Keyword(KW::If)) {
            cur_expr.push(self.parse_if());
        } else if matches!(next_token, TT::Keyword(KW::Try)) {
            cur_expr.push(self.parse_try());
        } else if matches!(next_token, TT::Keyword(KW::Loop)) {
            cur_expr.push(self.parse_loop());
        } else if matches!(next_token, TT::Keyword(KW::Throw)) {
            cur_expr.push(self.parse_throw());
        } else if matches!(next_token, TT::Operator(Operator::LCurly)) {
            cur_expr.push(self.parse_block());
        } else if matches!(next_token, TT::Operator(Operator::LBracket)) {
            cur_expr.push(self.parse_list());
        } else if matches!(next_token, TT::Str(_)) {
            cur_expr.push(self.parse_str());
        } else if matches!(next_token, TT::Keyword(KW::Func) | TT::Keyword(KW::Class)) {
            cur_expr.push(self.parse_func());
        } else if next_token.is_unary_op() {
            cur_expr.push(self.parse_unop());
        } else if next_token.is_value() {
            if !self.can_peek_ahead(1) {
                self.consume();
                cur_expr.push(next_token.to_value().unwrap());
            } else if self.peek_ahead(1).info.is_binary_op() {
                cur_expr.push(self.parse_binop());
            } else if self.peek_ahead(1).info == TT::Operator(Operator::LParen) {
                cur_expr.push(self.parse_fn_call());
            } else if self.peek_ahead(1).info.is_semi() {
                self.advance(2);
                cur_expr.push(next_token.to_value().unwrap());
            } else {
                panic!("{next_token:?} -- {:?}", self.peek_ahead(1).info);
            }
        } else {
            panic!("Invalid sequence: {next_token:?}");
        }
        Some(cur_expr)
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }
    fn can_peek_ahead(&self, n: usize) -> bool {
        self.current + n < self.tokens.len()
    }
    fn can_peek(&self) -> bool {
        self.current < self.tokens.len()
    }
    fn peek_ahead(&self, n: usize) -> &Token {
        &self.tokens[self.current + n]
    }
    // fn previous(&self) -> &Token {
    // &self.tokens[self.current - 1]
    // }
    fn consume(&mut self) -> &Token {
        self.current += 1;
        &self.tokens[self.current - 1]
    }
    fn advance(&mut self, n: usize) {
        self.current += n;
    }
    fn is_finished(&self) -> bool {
        self.current == self.tokens.len()
    }

    fn parse_var_dec(&mut self) -> Expression {
        type KW = Keyword;
        type TT = TokenType;
        type Expr = Expression;
        // type Op = Operator;

        assert!(matches!(
            self.peek().info,
            TT::Keyword(KW::Let) | TT::Keyword(KW::Var)
        ));
        let maker_kw = self.consume();
        let config = VarData {
            is_const: matches!(maker_kw.info, TT::Keyword(KW::Let)),
            is_init: false,
            value: Default::default(),
        };

        let TT::Id(ref var_name) = self.peek().info else {
            panic!(
                "(line {}) variable creation followed by non-identifier({:?}) is disallowed",
                self.peek().metadata.line_number,
                self.peek().info
            );
        };

        Expr::MakeVar(var_name.clone(), config)
    }

    fn parse_if(&mut self) -> Expression {
        type KW = Keyword;
        type TT = TokenType;

        assert!(matches!(
            self.peek().info,
            TT::Keyword(KW::If) | TT::Keyword(KW::Elif)
        ));
        let _if = self.consume();
        let cond_expr_len = self.tokens[self.current..]
            .iter()
            .take_while(|t| t.info != TT::Operator(Operator::LCurly))
            .count();

        let condition = &self.tokens[self.current..(self.current + cond_expr_len)];
        // dbg!(&condition);

        let condition_parser = Parser::new(condition);
        self.advance(cond_expr_len);

        let body = self.parse_block();

        let to_else = if !self.can_peek() {
            Expression::default()
        } else if self.peek().info == TT::Keyword(KW::Else) {
            let _else = self.consume();
            self.parse_block()
        } else if self.peek().info == TT::Keyword(KW::Elif) {
            self.parse_if()
        } else {
            Expression::default()
        };

        Expression::If(IfExpr {
            condition: Box::new(condition_parser.parse()),
            to_resolve: Box::new(body),
            to_else: Box::new(to_else),
        })
    }
    fn parse_try(&mut self) -> Expression {
        type KW = Keyword;
        type TT = TokenType;

        assert!(matches!(self.peek().info, TT::Keyword(KW::Try)));
        let _try = self.consume();
        let body = self.parse_block();

        let catch = if !self.can_peek() {
            Expression::default()
        } else if self.peek().info == TT::Keyword(KW::Catch) {
            let _catch = self.consume();
            self.parse_block()
        } else {
            Expression::default()
        };

        Expression::Try(TryExpr {
            body: Box::new(body),
            catch: Box::new(catch),
        })
    }
    fn parse_loop(&mut self) -> Expression {
        type KW = Keyword;
        type TT = TokenType;

        assert!(matches!(self.peek().info, TT::Keyword(KW::Loop)));
        let _loop = self.consume();
        let cond_expr_len = self.tokens[self.current..]
            .iter()
            .take_while(|t| t.info != TT::Operator(Operator::LCurly))
            .count();

        let condition = &self.tokens[self.current..(self.current + cond_expr_len)];

        let condition_parser = Parser::new(condition);
        self.advance(cond_expr_len);

        let body = self.parse_block();

        Expression::Loop(LoopExpr {
            condition: Box::new(condition_parser.parse()),
            to_resolve: Box::new(body),
        })
    }
    fn parse_throw(&mut self) -> Expression {
        type KW = Keyword;
        type TT = TokenType;
        // type Expr = Expression;

        assert!(matches!(self.peek().info, TT::Keyword(KW::Throw)));
        let _throw = self.consume().clone();
        let value = self.parse_expr().unwrap_or_default();
        Expression::Throw(Box::new(value))
    }
    fn parse_unop(&mut self) -> Expression {
        type TT = TokenType;

        assert!(self.peek().info.is_unary_op());
        let op = match self.consume().info {
            TT::Operator(Operator::Not) => Operator::Not,
            TT::Operator(Operator::Minus) => Operator::Minus,
            _ => unreachable!(),
        };

        Expression::UnOp(UnOpExpr {
            op,
            operand: Box::new(self.parse_expr().unwrap()),
        })
    }
    fn parse_binop(&mut self) -> Expression {
        type Op = Operator;
        type Expr = Expression;

        assert!(self.peek().info.is_value() && self.peek_ahead(1).info.is_binary_op());
        let left = self.consume().info.to_value().unwrap();
        let mut op = self.consume().info.to_op().unwrap();
        let mut right = self.parse_expr().unwrap();

        // Syntax sugar on operators
        if op == Op::Minus {
            right = Expr::UnOp(UnOpExpr {
                op,
                operand: Box::new(right),
            });
            op = Op::Plus;
        } else if op == Op::NEqual {
            let equal_to = Expr::BinOp(BinOpExpr {
                op: Op::Equals,
                left: Box::new(left),
                right: Box::new(right),
            });
            return Expr::UnOp(UnOpExpr {
                op: Op::Not,
                operand: Box::new(equal_to),
            });
        } else if op == Operator::GrEq {
            let less_than = Expr::BinOp(BinOpExpr {
                op: Op::Lt,
                left: Box::new(left),
                right: Box::new(right),
            });
            return Expr::UnOp(UnOpExpr {
                op: Op::Not,
                operand: Box::new(less_than),
            });
        } else if op == Op::LtEq {
            let less_than = Expr::BinOp(BinOpExpr {
                op: Op::Lt,
                left: Box::new(left.clone()),
                right: Box::new(right.clone()),
            });
            let equal_to = Expr::BinOp(BinOpExpr {
                op: Op::Equals,
                left: Box::new(left),
                right: Box::new(right),
            });
            return Expr::BinOp(BinOpExpr {
                op: Op::Or,
                left: Box::new(less_than),
                right: Box::new(equal_to),
            });
        } else if matches!(op, Op::PlusEq | Op::MinusEq | Op::StarEq | Op::SlashEq) {
            op = match op {
                Op::PlusEq => Op::Plus,
                Op::MinusEq => Op::Minus,
                Op::StarEq => Op::Star,
                Op::SlashEq => Op::Slash,
                _ => unreachable!(),
            };
            let result = if op != Op::Minus {
                Expr::BinOp(BinOpExpr {
                    op,
                    left: Box::new(left.clone()),
                    right: Box::new(right),
                })
            } else {
                Expr::BinOp(BinOpExpr {
                    op: Op::Plus,
                    left: Box::new(left.clone()),
                    right: Box::new(Expr::UnOp(UnOpExpr {
                        op: Op::Minus,
                        operand: Box::new(right),
                    })),
                })
            };
            return Expr::BinOp(BinOpExpr {
                op: Op::Assign,
                left: Box::new(left),
                right: Box::new(result),
            });
        }

        Expr::BinOp(BinOpExpr {
            op,
            left: Box::new(left),
            right: Box::new(right),
        })
    }

    fn parse_block(&mut self) -> Expression {
        assert!(self.peek().info == TokenType::Operator(Operator::LCurly));
        self.consume();

        let block_start = self.current;

        let mut lcurlys_rem = 1;
        while !self.is_finished() && lcurlys_rem > 0 {
            match self.consume().info {
                TokenType::Operator(Operator::RCurly) => lcurlys_rem -= 1,
                TokenType::Operator(Operator::LCurly) => lcurlys_rem += 1,
                _ => (),
            };
        }
        // if self.is_finished() {
        // panic!("Unclosed block");
        // }
        let body = &self.tokens[block_start..(self.current - 1)];
        let body_parser = Parser::new(body);
        // dbg!(&body);
        // dbg!(self);
        body_parser.parse()
    }
    fn parse_func(&mut self) -> Expression {
        type TT = TokenType;
        type KW = Keyword;
        type Op = Operator;
        use crate::obj::Object;

        assert!(matches!(
            self.peek().info,
            TT::Keyword(KW::Func) | TT::Keyword(KW::Class)
        ));
        let is_class = self.consume().info == TT::Keyword(KW::Class);

        assert!(self.peek().info == TT::Operator(Op::LParen));
        let _lparen = self.consume();
        let params_it = self
            .tokens
            .iter()
            .skip(self.current)
            .take_while(|t| t.info != TT::Operator(RParen));
        let mut params = vec![];
        let mut should_be_comma = false;
        for _ in params_it {
            let p = self.consume();
            match &p.info {
                TT::Id(name) => {
                    if !should_be_comma {
                        if name == "recurs" {
                            panic!(
                                "(Line {}) Invalid function parameter name: recurs",
                                p.metadata.line_number
                            )
                        }
                        params.push(name.clone());
                        should_be_comma = true;
                    } else {
                        panic!(
                            "(Line {}) Identifer where comma should be in function parameters",
                            p.metadata.line_number
                        );
                    }
                }
                TT::Operator(Op::Comma) => {
                    if !should_be_comma {
                        panic!(
                            "(Line {}) Comma in bad spot in function parameters",
                            p.metadata.line_number
                        );
                    }
                    should_be_comma = false;
                }

                tok => panic!(
                    "(Line {}) Bad token({tok:?}) in function parameters",
                    p.metadata.line_number
                ),
            }
        }
        let _rparen = self.consume();
        let mut body = self.parse_block();
        if is_class {
            let mut prefix =
                Expression::MakeVar("self".into(), VarData::make_constant(Object::make_inst()));

            prefix.push(body);
            prefix.push(Expression::Variable("self".into()));
            body = prefix;
        }

        Object::make_func(params, false, Box::new(body)).make_expr()
    }
    fn parse_fn_call(&mut self) -> Expression {
        type TT = TokenType;

        assert!(matches!(self.peek().info, TT::Id(_)));
        if !matches!(self.peek().info, TT::Id(_)) {
            panic!("A function call can only be performed on a variable")
        }
        let TT::Id(func) = self.peek().info.clone() else {
            let pk = self.peek();
            panic!(
                "(Line {}) Can not call non-variable ({:?})",
                pk.metadata.line_number, pk.info
            );
        };
        self.consume();

        let args = self.parse_args();

        let mut fn_data = vec![Expression::Variable(func)];
        fn_data.extend(args);

        Expression::Call(fn_data)
    }

    fn parse_args(&mut self) -> Vec<Expression> {
        self.parse_collection(Operator::LParen, Operator::RParen)
    }
    fn parse_list(&mut self) -> Expression {
        let vals = self.parse_collection(Operator::LBracket, Operator::RBracket);
        Expression::ListExpr(vals)
    }
    fn parse_collection(&mut self, opening: Operator, closing: Operator) -> Vec<Expression> {
        type TT = TokenType;
        type Op = Operator;

        assert!(self.peek().info == TT::Operator(opening));
        let _opening = self.consume();
        let mut vec = vec![];
        while self.can_peek() && self.peek().info != TT::Operator(closing) {
            let start = self.current;
            let mut nest_level = 0;
            while self.can_peek() {
                if nest_level == 0
                    && (self.peek().info == TT::Operator(closing)
                        || self.peek().info == TT::Operator(Op::Comma))
                {
                    break;
                }
                if let TT::Operator(op) = self.peek().info {
                    if op.is_opening() {
                        nest_level += 1;
                    } else if op.is_closing() {
                        nest_level -= 1;
                    }
                }
                self.consume();
            }
            let tokens = &self.tokens[start..self.current];
            let parser = Parser::new(tokens);
            vec.push(parser.parse());

            if self.can_peek() && self.peek().info == TT::Operator(Op::Comma) {
                let _comma = self.consume();
            }
        }
        if self.can_peek() {
            if self.peek().info == TT::Operator(Op::Comma) {
                self.consume();
            }
            if self.peek().info == TT::Operator(closing) {
                self.consume();
            }
        }
        vec
    }
    fn parse_str(&mut self) -> Expression {
        use crate::obj::Object;
        assert!(matches!(self.peek().info, TokenType::Str(_)));

        let TokenType::Str(ref string) = self.consume().info else {
            panic!();
        };
        Object::make_str(string).make_expr()
    }
}
