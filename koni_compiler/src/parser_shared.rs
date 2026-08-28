use std::rc::Rc;

use crate::{
    tokenizer::{Token, TokenKind as TK, TokenType}, types::{Diagnostic, DiagnosticKind, Info, InfoType, Span, TypeAnnotation},
};

pub struct TokenCursor {
    pub tokens: Vec<Token>,
    pub pos: usize,
    pub current: Option<Token>,
    pub previous: Option<Token>,
    pub fp: Rc<str>,
}

impl TokenCursor {
    pub fn advance(&mut self) -> Option<&Token> {
        if self.current.is_some() {
            self.previous = self.current.clone();
            self.pos += 1;
        }
        self.current = self.tokens.get(self.pos).cloned();
        self.current.as_ref()
    }
    pub fn skip_newline(&mut self) {
        while let Some(c) = &self.current
            && c.ttype == TokenType::Newline
        {
            self.advance();
        }
    }

    pub fn current_span(&self) -> Span {
        match &self.current {
            Some(v) => v.span.clone(),
            None => match self.tokens.last() {
                Some(v) => v.span.clone(),
                None => Span {
                    start: 0,
                    end: None,
                    fp: self.fp.clone(),
                    ln: 0,
                    col: 0,
                    endln: None,
                    endcol: None,
                },
            },
        }
    }
    pub fn previous_span(&self) -> Span {
        match &self.previous {
            Some(v) => v.span.clone(),
            None => match self.tokens.last() {
                Some(v) => v.span.clone(),
                None => Span {
                    start: 0,
                    end: None,
                    fp: self.fp.clone(),
                    ln: 0,
                    col: 0,
                    endln: None,
                    endcol: None,
                },
            },
        }
    }
    pub fn expect_token(&self) -> Result<&Token, Diagnostic> {
        match &self.current {
            Some(v) => Ok(v),
            None => Err(Diagnostic {
                kind: DiagnosticKind::UnexpectedEOF,
                span: self.current_span(),
                info: vec![],
            }),
        }
    }

    pub fn eat(&mut self, ttype: TK) -> Result<Token, Diagnostic> {
        self.eat_info(ttype, vec![])
    }
    pub fn peek(&self, amnt: usize) -> Option<&Token> {
        self.tokens.get(self.pos + amnt)
    }

    pub fn eat_info(&mut self, ttype: TK, info: Vec<Info>) -> Result<Token, Diagnostic> {
        match self.current.clone() {
            None => Err(Diagnostic {
                kind: DiagnosticKind::UnexpectedEOF,
                span: self.current_span(),
                info: vec![],
            }),
            Some(v) => {
                if TK::from(&v.ttype) == ttype {
                    let token_payload = v.clone();
                    self.advance();
                    Ok(token_payload)
                } else {
                    self.advance();
                    Err(Diagnostic {
                        kind: DiagnosticKind::UnexpectedToken {
                            token: v.clone(),
                            expected: Some(ttype),
                        },
                        span: self.current_span(),
                        info,
                    })
                }
            }
        }
    }
}

pub trait TypeParsing {
    fn cursor(&mut self) -> &mut TokenCursor;

    fn parse_type(&mut self, generics: &[String]) -> Result<TypeAnnotation, Diagnostic> {
        let mut ty = self.parse_type_base(generics)?;

        while let Some(t) = &self.cursor().current
            && t.ttype == TokenType::LBracket
        {
            self.cursor().advance();
            self.cursor().eat(TK::RBracket)?;
            ty = TypeAnnotation::Array(Box::new(ty));
        }
        return Ok(ty);
    }
    fn type_parser_help(
        &mut self,
        generics: &[String],
        expected_amnt: Option<usize>,
        t: String,
    ) -> Result<Vec<TypeAnnotation>, Diagnostic> {
        let start_span = self.cursor().current_span();
        let mut args = vec![];
        if let Some(tk) = &self.cursor().peek(1)
            && tk.ttype == TokenType::LT
        {
            self.cursor().advance();
            self.cursor().skip_newline();
            args.push(self.parse_type(generics)?);
            while self.cursor().expect_token()?.ttype == TokenType::Comma {
                self.cursor().advance();
                self.cursor().skip_newline();
                if self.cursor().expect_token()?.ttype == TokenType::GT {
                    break;
                }
                args.push(self.parse_type(generics)?);
            }
            self.cursor().eat(TK::GT)?;
        }
        if let Some(amnt) = expected_amnt
            && amnt != args.len()
        {
            return Err(Diagnostic {
                kind: DiagnosticKind::InvalidTypeArgCount {
                    t,
                    expected: amnt,
                    got: args.len(),
                },
                span: start_span.merge(self.cursor().previous_span()),
                info: vec![Info {
                    msg: format!(
                        "this type requires {amnt} args, but it received {}",
                        args.len()
                    ),
                    span: Some(start_span),
                    itype: InfoType::Help,
                }],
            });
        }
        Ok(args)
    }

    fn parse_type_base(&mut self, generics: &[String]) -> Result<TypeAnnotation, Diagnostic> {
        match self.cursor().expect_token()?.ttype.clone() {
            TokenType::LParen => {
                self.cursor().advance();
                match &self.cursor().expect_token()?.ttype {
                    TokenType::RParen => {
                        self.cursor().advance();
                        return Ok(TypeAnnotation::Unit);
                    }
                    _ => {
                        let mut types = vec![self.parse_type(generics)?];
                        while self.cursor().expect_token()?.ttype == TokenType::Comma {
                            if let Some(t) = self.cursor().advance()
                                && t.ttype == TokenType::RParen
                            {
                                break;
                            }
                            types.push(self.parse_type(generics)?)
                        }
                        self.cursor().eat(TK::RParen)?;
                        return Ok(TypeAnnotation::Tuple(types));
                    }
                }
            }
            TokenType::Identifier { name } => match name.as_str() {
                "array" => Ok(TypeAnnotation::Array(Box::new(
                    self.type_parser_help(generics, Some(1), String::from("array"))?[0].clone(),
                ))),
                "dict" => {
                    let args = self.type_parser_help(generics, Some(2), String::from("dict"))?;
                    Ok(TypeAnnotation::Dict(
                        Box::new(args[0].clone()),
                        Box::new(args[1].clone()),
                    ))
                }
                _ => {
                    self.cursor().advance();
                    Ok(match name.as_str() {
                        "int" => TypeAnnotation::Int,
                        "float" => TypeAnnotation::Float,
                        "str" => TypeAnnotation::String,
                        "bool" => TypeAnnotation::Bool,
                        _ => {
                            if generics.contains(&name) {
                                TypeAnnotation::Generic(name.to_string())
                            } else {
                                let args =
                                    self.type_parser_help(generics, None, name.to_owned())?;
                                TypeAnnotation::User(name.to_string(), args)
                            }
                        }
                    })
                }
            },
            TokenType::Func => {
                let mut args = vec![];
                self.cursor().advance();
                self.cursor().eat(TK::LParen)?;

                if TokenType::RParen != self.cursor().expect_token()?.ttype {
                    args.push(self.parse_type(generics)?);
                    while TokenType::Comma == self.cursor().expect_token()?.ttype {
                        self.cursor().advance();
                        if TokenType::RParen != self.cursor().expect_token()?.ttype {
                            args.push(self.parse_type(generics)?);
                        }
                    }
                }
                self.cursor().eat(TK::RParen)?;
                let mut ret = TypeAnnotation::Unit;
                if let Some(t) = &self.cursor().current
                    && t.ttype == TokenType::ArrowLR
                {
                    self.cursor().advance();
                    ret = self.parse_type(generics)?;
                }
                Ok(TypeAnnotation::Func(args, Box::new(ret)))
            }
            _ => {
                todo!("{:?}", self.cursor().current)
            }
        }
    }
}
