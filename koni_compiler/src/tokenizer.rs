use crate::types::{Diagnostic, DiagnosticKind, Info, InfoType, Span};
use phf::phf_map;
use std::{fmt::Display, rc::Rc};
use strum::EnumDiscriminants;
#[derive(Clone, Debug)]
pub struct Token {
    pub span: Span,
    pub ttype: TokenType,
}

impl Token {
    pub fn as_identifier(&self) -> &String {
        match &self.ttype {
            TokenType::Identifier { name } => name,
            _ => unreachable!("isn't an identifier, {:?}", self.ttype),
        }
    }
    pub fn as_int(&self) -> i32 {
        match &self.ttype {
            TokenType::Int { val } => *val,
            _ => unreachable!("isn't an identifier, {:?}", self.ttype),
        }
    }
}
impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", <&TokenType as Into<TokenKind>>::into(&self.ttype))
    }
}
#[derive(Debug, Clone, EnumDiscriminants, PartialEq)]
#[strum_discriminants(name(TokenKind))]
pub enum TokenType {
    String { val: String },
    Int { val: i32 },
    Float { val: f64 },
    Plus,
    Minus,
    Multiply,
    Divide,
    Power,
    EqualsTo,
    NotEqualsTo,
    Newline,
    Identifier { name: String },
    If,
    Else,
    While,
    Const,
    Let,
    Func,
    LT,
    GT,
    LTE,
    GTE,
    Dot,
    LBracket,
    RBracket,
    LParen,
    RParen,
    Comma,
    Assign,
    ArrowLR,
    True,
    False,
    Colon,
    Not,
    Or,
    And,
    LBrace,
    RBrace,
    FatArrow,
    AtRate,
    Require,
    Version,
    DoubleColon,
    Return,
    Import,
    As,
    Struct,
    Enum,
    Mod,
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenKind::Else => write!(f, "else"),
            TokenKind::String => write!(f, "string"),
            TokenKind::EqualsTo => write!(f, "=="),
            TokenKind::Assign => write!(f, "="),
            TokenKind::Float => write!(f, "float"),
            TokenKind::Identifier => write!(f, "identifier"),
            TokenKind::NotEqualsTo => write!(f, "!="),
            TokenKind::Plus => write!(f, "+"),
            TokenKind::Int => write!(f, "int"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Newline => write!(f, "newline"),
            TokenKind::Const => write!(f, "const"),
            TokenKind::While => write!(f, "while"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Let => write!(f, "let"),
            TokenKind::Func => write!(f, "func"),
            TokenKind::LT => write!(f, "<"),
            TokenKind::GT => write!(f, ">"),
            TokenKind::LTE => write!(f, "<="),
            TokenKind::GTE => write!(f, ">="),
            TokenKind::Dot => write!(f, "."),
            TokenKind::LBracket => write!(f, "["),
            TokenKind::RBracket => write!(f, "]"),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::ArrowLR => write!(f, "->"),
            TokenKind::True => write!(f, "true"),
            TokenKind::False => write!(f, "false"),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Power => write!(f, "**"),
            TokenKind::Multiply => write!(f, "*"),
            TokenKind::Divide => write!(f, "/"),
            TokenKind::Or => write!(f, "or"),
            TokenKind::And => write!(f, "and"),
            TokenKind::Not => write!(f, "not"),
            TokenKind::FatArrow => write!(f, "=>"),
            TokenKind::AtRate => write!(f, "@"),
            TokenKind::Require => write!(f, "require"),
            TokenKind::Version => write!(f, "ver"),
            TokenKind::DoubleColon => write!(f, "::"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::Import => write!(f, "import"),
            TokenKind::As => write!(f, "as"),
            TokenKind::Struct => write!(f, "struct"),
            TokenKind::Enum => write!(f, "enum"),
            TokenKind::Mod => write!(f, "%")
        }
    }
}
pub struct Tokenizer {
    pub fp: Rc<str>,
    pub chars: Vec<char>,
    pub current: Option<char>,
    pub line: usize,
    pub col: usize,
    pub pos: usize,
    pub abort: bool,
}
static TOKEN_MAP: phf::Map<&'static str, TokenType> = phf_map! {
    "while" => TokenType::While,
    "if" => TokenType::If,
    "return" => TokenType::Return,
    "else" => TokenType::Else,
    "const" => TokenType::Const,
    "let" => TokenType::Let,
    "func" => TokenType::Func,
    "<" => TokenType::LT,
    ">" => TokenType::GT,
    "<=" => TokenType::LTE,
    ">=" => TokenType::GTE,
    "==" => TokenType::EqualsTo,
    "!=" => TokenType::NotEqualsTo,
    "+" => TokenType::Plus,
    "-" => TokenType::Minus,
    "." => TokenType::Dot,
    "(" => TokenType::LParen,
    ")" => TokenType::RParen,
    "=" => TokenType::Assign,
    "->" => TokenType::ArrowLR,
    "," => TokenType::Comma,
    "[" => TokenType::LBracket,
    "]" => TokenType::RBracket,
    "true" => TokenType::True,
    "false" => TokenType::False,
    "**" => TokenType::Power,
    "/" => TokenType::Divide,
    "*" => TokenType::Multiply,
    "not" => TokenType::Not,
    "or" => TokenType::Or,
    "and" => TokenType::And,
    ":" => TokenType::Colon,
    "{" => TokenType::LBrace,
    "}" => TokenType::RBrace,
    "\n" => TokenType::Newline,
    "=>" => TokenType::FatArrow,
    "@" => TokenType::AtRate,
    "require" => TokenType::Require,
    "ver" => TokenType::Version,
    "::" => TokenType::DoubleColon,
    "import" => TokenType::Import,
    "as" => TokenType::As,
    "struct" => TokenType::Struct,
    "enum" => TokenType::Enum,
    "%" => TokenType::Mod
};

impl Iterator for Tokenizer {
    type Item = Result<Token, Diagnostic>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.abort {
            return None;
        }
        self.skip_whitespace();
        if self.check_str() {
            let mut multiline = false;
            let mut raw = false;
            loop {
                match self.current {
                    None => unreachable!(),
                    Some(v) => match v {
                        'm' => multiline = true,
                        'r' => raw = true,
                        _ => break,
                    },
                }
            }
            let Some(c) = self.current else {
                unreachable!()
            };
            self.advance(1);
            let t = match c {
                '\'' => StrType::SingleQuote,
                '"' => StrType::DoubleQuote,
                _ => unreachable!(),
            };
            return Some(self.lex_str(t, multiline, raw));
        }
        let cspan = self.current_span();
        match self.current {
            Some(v) => {
                let t = self.match_token();
                if let Some(t) = t {
                    return Some(Ok(Token {
                        ttype: t,
                        span: cspan.merge(self.current_span()),
                    }));
                } else if v.is_numeric() {
                    return Some(self.lex_numeric());
                } else if v.is_alphabetic() || v == '_' {
                    let mut name = String::new();
                    while let Some(c) = self.current
                        && (c.is_alphanumeric() || c == '_')
                    {
                        name.push(c);
                        self.advance(1);
                    }
                    return Some(Ok(Token {
                        span: cspan.merge(self.current_span()),
                        ttype: TokenType::Identifier { name },
                    }));
                } else if v == '#' {
                    let span = self.current_span();
                    while let Some(c) = self.current {
                        self.advance(1);
                        if c == '\n' {
                            break;
                        }
                    }
                    return Some(Ok(Token {
                        ttype: TokenType::Newline,
                        span,
                    }));
                } else {
                    self.abort = true;
                    return Some(Err(Diagnostic {
                        kind: DiagnosticKind::UnexpectedChar { ch: v },
                        span: self.current_span(),
                        info: vec![],
                    }));
                }
            }
            None => return None,
        }
    }
}
impl Span {
    pub fn single(fp: Rc<str>, pos: usize, line: usize, col: usize) -> Self {
        Self {
            fp,
            start: pos,
            end: None,
            ln: line,
            col,
            endln: None,
            endcol: None,
        }
    }
    pub fn eof(fp: Rc<str>, ln: usize, col: usize) -> Self {
        Self {
            fp,
            start: 0,
            end: None,
            ln,
            col,
            endln: None,
            endcol: None,
        }
    }
    pub fn merge(&self, s2: Span) -> Span {
        match s2.endln {
            Some(_) => Self {
                fp: self.fp.clone(),
                start: self.start,
                end: s2.end,
                ln: self.ln,
                col: self.col,
                endln: s2.endln,
                endcol: s2.endcol,
            },
            None => Self {
                fp: self.fp.clone(),
                start: self.start,
                end: Some(s2.start),
                ln: self.ln,
                col: self.col,
                endln: Some(s2.ln),
                endcol: Some(s2.col),
            },
        }
    }
}

enum StrType {
    SingleQuote,
    DoubleQuote,
}
impl StrType {
    fn ch(&self) -> char {
        match self {
            Self::SingleQuote => '\'',
            Self::DoubleQuote => '"',
        }
    }
}
impl<'a> Tokenizer {
    pub fn new(input: &'a str, fp: Rc<str>) -> Self {
        let ci: Vec<char> = input.chars().collect();
        let mut t = Self {
            chars: ci,
            fp,
            current: None,
            line: 0,
            col: 0,
            pos: 0,
            abort: false,
        };
        t.current = t.peek_next(0);
        t
    }
    fn current_span(&self) -> Span {
        Span::single(self.fp.clone(), self.pos, self.line, self.col)
    }
    fn check_str(&self) -> bool {
        let mut i = 0;
        loop {
            match self.peek_next(i) {
                None => return false,
                Some(v) => match v {
                    'm' | 'r' => i += 1,
                    '\'' | '"' => return true,
                    _ => return false,
                },
            }
        }
    }
    fn match_token(&mut self) -> Option<TokenType> {
        let max_lookahead = 6;

        for len in (1..=max_lookahead).rev() {
            if self.pos + len <= self.chars.len() {
                let slice = &self.chars[self.pos..self.pos + len];
                let lookup_str: String = slice.iter().collect();
                if let Some(token_type) = TOKEN_MAP.get(lookup_str.as_str()) {
                    self.advance(len);
                    return Some(token_type.clone());
                }
            }
        }

        None
    }
    fn advance(&mut self, amnt: usize) -> Option<char> {
        let mut last_ch = None;
        for _ in 0..amnt {
            let cch = self.current?;

            if cch == '\n' {
                self.line += 1;
                self.col = 0;
            } else {
                self.col += 1;
            }

            last_ch = Some(cch);
            self.pos += 1;
            self.current = self.chars.get(self.pos).copied();
        }

        last_ch
    }
    fn peek_next(&self, amnt: usize) -> Option<char> {
        self.chars.get(self.pos + amnt).copied()
    }
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current {
            if !c.is_whitespace() || c == '\n' {
                break;
            }
            self.advance(1);
        }
    }
    fn eof_span(&self) -> Span {
        Span::eof(self.fp.clone(), self.line, self.col)
    }
    fn expect_char(&self) -> Result<char, Diagnostic> {
        match self.current {
            Some(v) => Ok(v),
            None => Err(Diagnostic {
                kind: DiagnosticKind::UnexpectedEOF,
                span: self.eof_span(),
                info: vec![],
            }),
        }
    }
    fn parse_escape_seq(&mut self, _stype: &StrType) -> String {
        unimplemented!()
    }
    fn lex_numeric(&mut self) -> Result<Token, Diagnostic> {
        let mut value = "".to_owned();
        let start_span = self.current_span();
        let mut is_float = false;
        while self
            .current
            .is_some_and(|f| f.is_numeric() || f == '_' || (f == '.' && !is_float))
        {
            let c = self.expect_char()?;
            if c == '_' {
                self.advance(1);
                continue;
            }
            if c == '.' {
                self.advance(1);
                value.push('.');
                is_float = true;
            }
            value.push(c);
            self.advance(1);
        }
        if is_float {
            let val = value.parse::<f64>().map_err(|_| Diagnostic {
                kind: DiagnosticKind::InvalidFloat { float: value },
                span: start_span.merge(self.current_span()),
                info: vec![],
            })?;
            Ok(Token {
                span: start_span.merge(self.current_span()),
                ttype: TokenType::Float { val },
            })
        } else {
            let val = value.parse::<i32>().map_err(|_| Diagnostic {
                kind: DiagnosticKind::InvalidInt { int: value },
                span: start_span.merge(self.current_span()),
                info: vec![],
            })?;
            Ok(Token {
                span: start_span.merge(self.current_span()),
                ttype: TokenType::Int { val },
            })
        }
    }
    fn lex_str(&mut self, stype: StrType, multiline: bool, raw: bool) -> Result<Token, Diagnostic> {
        let mut value = "".to_owned();
        let start_span = self.current_span();

        while let Some(char) = self.current
            && char != stype.ch()
        {
            if !multiline && char == '\n' {
                break;
            }
            if char == '\\' && !raw {
                self.advance(1);
                value += &self.parse_escape_seq(&stype)
            } else {
                value.push(char)
            }
            self.advance(1);
        }
        let mut unterminated = false;
        if let Some(char) = self.current
            && char != stype.ch()
        {
            unterminated = true;
        }
        if self.current.is_none() {
            unterminated = true;
        }
        if unterminated {
            return Err(Diagnostic {
                kind: DiagnosticKind::UnterminatedStringLiteral,
                span: start_span.merge(self.current_span()),
                info: vec![Info {
                    msg: String::from("string started here"),
                    span: Some(start_span),
                    itype: InfoType::Help,
                }],
            });
        }
        self.advance(1);
        Ok(Token {
            span: start_span.merge(self.current_span()),
            ttype: TokenType::String { val: value },
        })
    }
}
