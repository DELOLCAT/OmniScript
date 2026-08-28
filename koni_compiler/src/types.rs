use crate::tokenizer::{Token, TokenKind};
use std::{
    fmt::{Debug, Display},
    rc::Rc,
};
use serde::Serialize;

fn serialize_rc_str<S>(rc: &Rc<str>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(rc)
}

#[derive(Debug, Clone, Serialize)]
pub struct Span {
    #[serde(serialize_with = "serialize_rc_str")]
    pub fp: Rc<str>,
    pub start: usize,
    pub end: Option<usize>,
    pub ln: usize,
    pub col: usize,
    pub endln: Option<usize>,
    pub endcol: Option<usize>,
}
pub enum Type {
    Int,
    Float,
    String,
    Bool,
    Unit,

    Param { name: String, idx: usize },
    Struct { name: String, args: Vec<Type> },
    Enum { name: String, args: Vec<Type> },
    Func(Vec<Type>, Box<Type>),
    Array(Box<Type>),
    Dict(Box<Type>, Box<Type>),
}
#[derive(Debug, Clone, Serialize)]
pub enum TypeAnnotation {
    Int,
    Float,
    String,
    Bool,
    Unit,
    Func(Vec<TypeAnnotation>, Box<TypeAnnotation>),
    Array(Box<TypeAnnotation>),
    Dict(Box<TypeAnnotation>, Box<TypeAnnotation>),
    Generic(String),
    User(String, Vec<TypeAnnotation>),
    Tuple(Vec<TypeAnnotation>)
}
impl Display for TypeAnnotation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeAnnotation::Int => write!(f, "int"),
            TypeAnnotation::Float => write!(f, "float"),
            TypeAnnotation::String => write!(f, "str"),
            TypeAnnotation::Bool => write!(f, "bool"),
            TypeAnnotation::Unit => write!(f, "()"),
            TypeAnnotation::Func(args, ret) => {
                let mut out = String::from("func(");
                for (idx, arg) in args.iter().enumerate() {
                    out.push_str(format!("{}", arg).as_str());
                    if idx != args.len() - 1 {
                        out.push_str(", ");
                    }
                }
                out.push_str(format!(") -> {}", ret).as_str());
                write!(f, "{out}")
            }
            TypeAnnotation::Array(t) => write!(f, "array<{t}>"),
            TypeAnnotation::Dict(k, v) => write!(f, "dict<{k}, {v}>"),
            TypeAnnotation::Generic(t) => write!(f, "{t}"),
            TypeAnnotation::User(t, args) => {
                let mut out = t.to_owned();
                if !args.is_empty() {
                    out.push('<');
                    for (idx, arg) in args.iter().enumerate() {
                        out.push_str(format!("{arg}").as_str());
                        if idx != args.len() - 1 {
                            out.push_str(", ")
                        }
                    }
                    out.push('>');
                }
                write!(f, "{out}")
            }
            TypeAnnotation::Tuple(type_annotations) => {
                write!(f, "({})", type_annotations.iter().map(|i| i.to_string()).collect::<Vec<_>>().join(", "))
            },
        }
    }
}
#[derive(Debug, Clone)]
pub struct Info {
    pub msg: String,
    pub span: Option<Span>,
    pub itype: InfoType,
}

impl Display for InfoType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InfoType::Help => write!(f, "help"),
            InfoType::_Hint => write!(f, "hint"),
            InfoType::Note => write!(f, "note"),
        }
    }
}

#[derive(Debug, Clone)]
pub enum InfoType {
    Help,
    Note,
    _Hint,
}

#[derive(Clone, Debug)]
pub enum Severity {
    Warn(WarnCode),
    Err(ErrCode),
}
#[derive(Clone, Debug)]
#[repr(u8)]
pub enum ErrCode {
    UnexpectedChar = 1,
    UnexpectedEOF,
    UnterminatedStringLiteral,
    InvalidInt,
    InvalidFloat,
    UnexpectedToken,
    InvalidTypeArgCount,
    DuplicateIndex,
}
#[derive(Debug, Clone)]
pub enum DiagnosticKind {
    UnexpectedChar {
        ch: char,
    },
    UnexpectedEOF,
    UnterminatedStringLiteral,
    InvalidInt {
        int: String,
    },
    InvalidFloat {
        float: String,
    },
    UnexpectedToken {
        token: Token,
        expected: Option<TokenKind>,
    },
    InvalidTypeArgCount {
        t: String,
        expected: usize,
        got: usize,
    },
    DuplicateIndex {
        idx: i32,
    },
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum WarnCode {
    ReqAdded = 1,
    FormatStrNoExpr,
    Unreachable,
    RedundantIfStatement,
}
impl Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Warn(c) => write!(f, "warn[W{:03}]", c.clone() as u8),
            Self::Err(c) => write!(f, "error[E{:03}]", c.clone() as u8),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub kind: DiagnosticKind,
    pub span: Span,
    pub info: Vec<Info>,
}

impl Diagnostic {
    pub fn severity(&self) -> Severity {
        match &self.kind {
            DiagnosticKind::DuplicateIndex { .. } => Severity::Err(ErrCode::DuplicateIndex),
            DiagnosticKind::UnexpectedChar { .. } => Severity::Err(ErrCode::UnexpectedChar),
            DiagnosticKind::UnexpectedEOF => Severity::Err(ErrCode::UnexpectedEOF),
            DiagnosticKind::UnterminatedStringLiteral => Severity::Err(ErrCode::UnterminatedStringLiteral),
            DiagnosticKind::InvalidInt { .. } => Severity::Err(ErrCode::InvalidInt),
            DiagnosticKind::InvalidFloat { .. } => Severity::Err(ErrCode::InvalidFloat),
            DiagnosticKind::UnexpectedToken { .. } => Severity::Err(ErrCode::UnexpectedToken),
            DiagnosticKind::InvalidTypeArgCount { .. } => Severity::Err(ErrCode::InvalidTypeArgCount),
        }
    }
}
impl std::fmt::Display for DiagnosticKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            DiagnosticKind::InvalidInt { int } => write!(f, "invalid int: {int}"),
            DiagnosticKind::UnexpectedChar { ch } => write!(f, "unexpected char: {ch}"),
            DiagnosticKind::UnexpectedEOF => write!(f, "unexpected end of file"),
            DiagnosticKind::UnterminatedStringLiteral => write!(f, "unterminated string literal"),
            DiagnosticKind::UnexpectedToken { token, expected } => match expected {
                None => write!(f, "unexpected token `{token}`"),
                Some(v) => write!(f, "expected token `{v}`, got token `{token}`"),
            },
            DiagnosticKind::InvalidFloat { float } => write!(f, "invalid float: {float}"),
            DiagnosticKind::InvalidTypeArgCount { t, expected, got } => write!(
                f,
                "invalid type argument count for `{t}`: expected {expected}, got {got}"
            ),
            DiagnosticKind::DuplicateIndex { idx } => write!(f, "duplicate index for {idx}"),
        }
    }
}
