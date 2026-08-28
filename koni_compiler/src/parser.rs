use crate::{
    parser_shared::{TokenCursor, TypeParsing}, tokenizer::{Token, TokenKind as TK, TokenType}, types::{
        Diagnostic, DiagnosticKind, Info, InfoType, Span,
        TypeAnnotation::{self},
    },
};
use serde::Serialize;
use std::{fmt::Debug, path::Path as StdPath, rc::Rc};
#[derive(Debug, Clone, Serialize)]
pub struct ASTNode {
    span: Span,
    pub node: ASTNodeType,
}
#[derive(Debug, Clone, Serialize)]
pub struct FunctionParameter {
    name: String,
    arg_type: TypeAnnotation,
}

#[derive(Debug, Clone, Serialize)]
enum IfElseBlock {
    _Block(Block),
    _ElseIf(If),
}

#[derive(Debug, Clone, Serialize)]
pub struct If {
    span: Span,
    condition: Box<ASTNode>,
    body: Box<Block>,
    else_body: Option<Box<IfElseBlock>>,
}
#[derive(Debug, Clone, Serialize)]
pub enum ASTNodeType {
    String(String),
    Int(i32),
    Float(f64),
    Variable(String),
    Attribute(Box<ASTNode>, String),
    Index(Box<ASTNode>, Box<ASTNode>),
    Declare(String, Option<Box<ASTNode>>, Option<TypeAnnotation>),
    Const(Const),
    Assign(String, Box<ASTNode>),
    Call(Box<ASTNode>, Vec<ASTNode>),
    Bool(bool),
    Unit,
    BinOp(Box<ASTNode>, BinOpType, Box<ASTNode>),
    UnaryOp(Box<ASTNode>, UnaryOpType),
    Function(Function),
    _If(If),
    _Block(Block),
    While {
        condition: Box<ASTNode>,
        body: Box<Block>,
    },
    Path(Path),
    Return {
        expr: Option<Box<ASTNode>>,
    },
    _Import(Import),
}

#[derive(Debug, Clone, Serialize)]
pub struct Block {
    span: Span,
    items: Vec<ASTNode>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Function {
    name: String,
    args: Vec<FunctionParameter>,
    ret: TypeAnnotation,
    body: Box<Block>,
    span: Span,
}

pub struct _ParsedFile {
    imports: Vec<Import>,
    functions: Vec<Function>,
    structs: Vec<Struct>,
    enums: Vec<Enum>,
    constants: Vec<Const>,
    // TODO: impls
}
#[derive(Debug)]
pub enum StartIRNode {
    Import(Import),
    Function(Function),
    Struct(Struct),
    Enum(Enum),
    Const(Const),
}
#[derive(Debug, Clone, Serialize)]
pub struct Const {
    name: String,
    value: Box<ASTNode>,
    ty: Option<TypeAnnotation>,
    span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub enum Struct {
    KeyValue {
        fields: Vec<StructField>,
        name: String,
        span: Span,
    },
    Tuple {
        name: String,
        fields: Vec<TypeAnnotation>,
        span: Span,
    },
}
#[derive(Debug, Clone, Serialize)]
pub struct StructField {
    name: String,
    ty: StructFieldType,
}

#[derive(Debug, Clone, Serialize)]
pub enum StructFieldType {
    Regular(TypeAnnotation),
    Struct(Vec<StructField>),
    Enum(Vec<EnumVariant>),
}
#[derive(Debug, Clone, Serialize)]
pub struct Enum {
    name: String,
    variants: Vec<EnumVariant>,
    span: Span,
}
#[derive(Debug, Clone, Serialize)]
pub struct EnumVariant {
    name: String,
    ty: EnumVariantType,
}
#[derive(Debug, Clone, Serialize)]
pub enum EnumVariantType {
    Regular,
    Tuple(Vec<TypeAnnotation>),
    Struct(Vec<StructField>),
    Enum(Vec<EnumVariant>),
}

#[derive(Debug, Clone, Serialize)]
pub struct Path {
    items: Vec<String>,
}
#[derive(Debug, Clone, Serialize)]
pub struct Import {
    path: Path,
    alias: Option<String>,
    children: Vec<Import>,
    span: Span,
}

#[derive(Debug, Clone, Serialize)]
pub enum UnaryOpType {
    Not,
    Negate,
}

#[derive(Debug, Clone, Serialize)]
pub enum BinOpType {
    Add,
    Subtract,
    Or,
    And,
    LT,
    LTE,
    GT,
    GTE,
    Multiply,
    Divide,
    Power,
    EqualsTo,
    NotEqualTo,
    Mod,
}
#[derive(Debug)]
pub struct Program {
    statements: Vec<ASTNode>,
}
pub struct Parser {
    cursor: TokenCursor,
    critical: bool,
    version: i32,      // |- for platform
    indexes: Vec<i32>, // |- files
}
impl Parser {
    pub fn new<P: AsRef<StdPath>>(tokens: Vec<Token>, fp: Rc<str>, _prelude_path: P) -> Self {
        let cursor = TokenCursor {
            tokens: tokens.clone(),
            pos: 0,
            current: tokens.first().cloned(),
            previous: None,
            fp,
        };
        Self {
            cursor,
            critical: false,
            version: 1,
            indexes: vec![],
        }
    }
}
impl Iterator for Parser {
    type Item = Result<StartIRNode, Diagnostic>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.critical {
            return None;
        }
        if self.cursor.current.is_none() {
            return None;
        }
        let v = self.item();
        if let Err(e) = self.cursor.eat(TK::Newline)
            && v.as_ref().is_some_and(|f| f.is_ok())
        {
            return Some(Err(e));
        }
        v
    }
}
impl TypeParsing for Parser {
    fn cursor(&mut self) -> &mut TokenCursor {
        &mut self.cursor
    }
}
impl Parser {
    pub fn expr(&mut self) -> Result<ASTNode, Diagnostic> {
        self.or()
    }
    fn type_parser_help(
        &mut self,
        generics: &[String],
        expected_amnt: Option<usize>,
        t: String,
    ) -> Result<Vec<TypeAnnotation>, Diagnostic> {
        let start_span = self.cursor.current_span();
        let mut args = vec![];
        if let Some(tk) = &self.cursor.peek(1)
            && tk.ttype == TokenType::LT
        {
            self.cursor.advance();
            self.cursor.skip_newline();
            args.push(self.parse_type(generics)?);
            while self.cursor.expect_token()?.ttype == TokenType::Comma {
                self.cursor.advance();
                self.cursor.skip_newline();
                if self.cursor.expect_token()?.ttype == TokenType::GT {
                    break;
                }
                args.push(self.parse_type(generics)?);
            }
            self.cursor.eat(TK::GT)?;
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
                span: start_span.merge(self.cursor.previous_span()),
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
    // TODO: anonymous functions
    // fn check_anon_fn(&mut self) -> bool {
    //     let paren = 0;
    //     while let Some(t) = self.cursor.current {
    //         match t.ttype {
    //             TokenType::Identifier { .. } => {
    //                 match self.cursor.peek(1) {
    //                     Some(v) => if v.ttype == TokenType::FatArrow {
    //                         return true
    //                     } else {
    //                         return false
    //                     }
    //                     None => return false
    //                 }
    //             },
    //             TokenType::LParen => {
    //                 let i = 1;
    //                 loop {
    //                     match self.cursor.peek(i) {
    //                         Some(v) => match v.ttype {
    //                             TokenType::Identifier { .. } => continue,
    //                             TokenType::Comma => continue,
    //                             TokenType::RParen => if self.cursor.peek(i+1).is_some_and(|t| t.ttype == TokenType::FatArrow) {
    //                                 return true
    //                             } else {
    //                                 return false
    //                             },
    //                             _ => return false
    //                         },
    //                         None => return false
    //                     }
    //                 }
    //             }
    //         }
    //     }
    //     false
    // }

    /// Parses a type in the format (type1, type2)
    ///
    /// # Arguments
    ///
    /// * `generics` - A list of generics
    /// * `dont_eat_lparen` - Controls weather it would eat the beginning `LParen`. Usually is false in most cases
    fn parse_tuple_type(
        &mut self,
        generics: &[String],
        dont_eat_lparen: bool,
    ) -> Result<Vec<TypeAnnotation>, Diagnostic> {
        if !dont_eat_lparen {
            self.cursor.eat(TK::LParen)?;
        }
        let mut types = vec![self.parse_type(generics)?];
        while self.cursor.expect_token()?.ttype == TokenType::Comma {
            if let Some(t) = self.cursor.advance()
                && t.ttype == TokenType::RParen
            {
                break;
            }
            types.push(self.parse_type(generics)?)
        }
        self.cursor.eat(TK::RParen)?;
        return Ok(types);
    }
    pub fn parse_type_base(&mut self, generics: &[String]) -> Result<TypeAnnotation, Diagnostic> {
        match self.cursor.expect_token()?.ttype.clone() {
            TokenType::LParen => {
                self.cursor.advance();
                match &self.cursor.expect_token()?.ttype {
                    TokenType::RParen => {
                        self.cursor.advance();
                        return Ok(TypeAnnotation::Unit);
                    }
                    _ => {
                        let mut types = vec![self.parse_type(generics)?];
                        while self.cursor.expect_token()?.ttype == TokenType::Comma {
                            if let Some(t) = self.cursor.advance()
                                && t.ttype == TokenType::RParen
                            {
                                break;
                            }
                            types.push(self.parse_type(generics)?)
                        }
                        self.cursor.eat(TK::RParen)?;
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
                    self.cursor.advance();
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
                self.cursor.advance();
                self.cursor.eat(TK::LParen)?;

                if TokenType::RParen != self.cursor.expect_token()?.ttype {
                    args.push(self.parse_type(generics)?);
                    while TokenType::Comma == self.cursor.expect_token()?.ttype {
                        self.cursor.advance();
                        if TokenType::RParen != self.cursor.expect_token()?.ttype {
                            args.push(self.parse_type(generics)?);
                        }
                    }
                }
                self.cursor.eat(TK::RParen)?;
                let mut ret = TypeAnnotation::Unit;
                if let Some(t) = &self.cursor.current
                    && t.ttype == TokenType::ArrowLR
                {
                    self.cursor.advance();
                    ret = self.parse_type(generics)?;
                }
                Ok(TypeAnnotation::Func(args, Box::new(ret)))
            }
            _ => {
                todo!("{:?}", self.cursor.current)
            }
        }
    }
    pub fn parse_type(&mut self, generics: &[String]) -> Result<TypeAnnotation, Diagnostic> {
        let mut ty = self.parse_type_base(generics)?;

        while let Some(t) = &self.cursor.current
            && t.ttype == TokenType::LBracket
        {
            self.cursor.advance();
            self.cursor.eat(TK::RBracket)?;
            ty = TypeAnnotation::Array(Box::new(ty));
        }
        return Ok(ty);
    }
    pub fn parse_func(
        &mut self,
        nobody: bool,
    ) -> Result<
        (
            Span,
            String,
            Vec<FunctionParameter>,
            TypeAnnotation,
            Option<Block>,
        ),
        Diagnostic,
    > {
        let start_span = self.cursor.current_span();
        self.cursor.advance();
        let name = self.cursor.eat(TK::Identifier)?.as_identifier().to_owned();
        let mut generics = vec![];
        let mut args = vec![];
        if self.cursor.expect_token()?.ttype == TokenType::LT {
            self.cursor.advance();
            let i = self.cursor.eat(TK::Identifier)?.as_identifier().to_owned();
            generics.push(i);
            while self.cursor.expect_token()?.ttype == TokenType::Comma {
                self.cursor.advance();
                if self.cursor.expect_token()?.ttype == TokenType::GT {
                    break;
                }
                let i = self.cursor.eat(TK::Identifier)?.as_identifier().to_owned();
                generics.push(i);
            }
            self.cursor.eat(TK::GT)?;
        }
        self.cursor.eat(TK::LParen)?;
        if self.cursor.expect_token()?.ttype != TokenType::RParen {
            let arg_name = self.cursor.eat(TK::Identifier)?.as_identifier().to_owned();
            self.cursor.eat(TK::Colon)?;
            let arg_type = self.parse_type(&generics)?;
            args.push(FunctionParameter {
                name: arg_name,
                arg_type,
            });

            while self.cursor.expect_token()?.ttype == TokenType::Comma {
                self.cursor.advance();
                let arg_name = self.cursor.eat(TK::Identifier)?.as_identifier().to_owned();
                self.cursor.eat(TK::Colon)?;
                let arg_type = self.parse_type(&generics)?;
                args.push(FunctionParameter {
                    name: arg_name,
                    arg_type,
                });
            }
        }
        let mut ret_type = TypeAnnotation::Unit;
        self.cursor.eat(TK::RParen)?;
        if let Some(t) = &self.cursor.current
            && t.ttype == TokenType::ArrowLR
        {
            self.cursor.advance();
            ret_type = self.parse_type(&generics)?;
        }
        let mut body = None;
        if !nobody {
            body = Some(self.block()?);
        }
        return Ok((
            start_span.merge(self.cursor.current_span()),
            name,
            args,
            ret_type,
            body,
        ));
    }
    fn block(&mut self) -> Result<Block, Diagnostic> {
        let start_span = self.cursor.current_span();
        self.cursor.eat(TK::LBrace)?;
        let mut st = vec![];
        while self.cursor.expect_token()?.ttype != TokenType::RBrace {
            self.cursor.skip_newline();
            if self.cursor.expect_token()?.ttype == TokenType::RBrace {
                break;
            }
            st.push(self.statement()?);
        }
        self.cursor.eat(TK::RBrace)?;
        Ok(Block {
            span: start_span.merge(self.cursor.previous_span()),
            items: st,
        })
    }
    fn parse_func_body(&mut self) -> Result<Function, Diagnostic> {
        let (span, name, args, ret, body) = self.parse_func(false)?;
        return Ok(Function {
            span,
            name,
            args,
            ret,
            body: Box::new(body.expect("(internal) no body for function")),
        });
    }
    fn parse_path_or_ident(&mut self) -> Result<ASTNode, Diagnostic> {
        let start_span = self.cursor.current_span();
        match self.cursor.current.clone() {
            Some(t) => match &t.ttype {
                TokenType::Identifier { name: n } => {
                    self.cursor.advance();
                    let mut path = vec![n.to_owned()];
                    while let Some(t) = &self.cursor.current
                        && t.ttype == TokenType::DoubleColon
                    {
                        self.cursor.advance();
                        let item = self.cursor.eat(TK::Identifier)?.as_identifier().to_owned();
                        path.push(item);
                    }
                    if path.len() > 1 {
                        return Ok(ASTNode {
                            span: start_span.merge(self.cursor.previous_span()),
                            node: ASTNodeType::Path(Path { items: path }),
                        });
                    }
                    return Ok(ASTNode {
                        span: start_span,
                        node: ASTNodeType::Variable(n.to_owned()),
                    });
                }
                _ => {
                    return Err(Diagnostic {
                        kind: DiagnosticKind::UnexpectedToken {
                            token: t,
                            expected: Some(TK::Identifier),
                        },
                        span: start_span.merge(self.cursor.previous_span()),
                        info: vec![],
                    });
                }
            },
            _ => todo!(),
        }
    }
    fn path(&mut self) -> Result<Path, Diagnostic> {
        let node = self.parse_path_or_ident()?;
        match node.node {
            ASTNodeType::Path(p) => Ok(p),
            ASTNodeType::Variable(n) => Ok(Path { items: vec![n] }),
            _ => unreachable!(),
        }
    }
    fn parse_enum_body(&mut self, generics: &[String]) -> Result<Vec<EnumVariant>, Diagnostic> {
        self.cursor.eat(TK::LBrace)?;
        self.cursor.skip_newline();

        let variant_name = self.cursor.eat(TK::Identifier)?.as_identifier().to_owned(); // TODO: warning against names that aren't UpperCamelCase
        let variant_type = self.parse_enum_type(&generics)?;
        let mut variants = vec![EnumVariant {
            name: variant_name,
            ty: variant_type,
        }];
        self.cursor.skip_newline();
        while let Some(t) = &self.cursor.current
            && t.ttype == TokenType::Comma
        {
            self.cursor.skip_newline();
            if self.cursor.expect_token()?.ttype == TokenType::RBrace {
                break;
            }
            let variant_name = self.cursor.eat(TK::Identifier)?.as_identifier().to_owned(); // TODO: warning against names that aren't UpperCamelCase
            let variant_type = self.parse_enum_type(&generics)?;
            variants.push(EnumVariant {
                name: variant_name,
                ty: variant_type,
            })
        }
        self.cursor.eat(TK::RBrace)?;
        return Ok(variants);
    }
    fn parse_enum(&mut self) -> Result<Enum, Diagnostic> {
        let start = self.cursor.current_span();
        self.cursor.eat(TK::Enum)?;
        let name = self.cursor.eat(TK::Identifier)?.as_identifier().to_owned();
        let generics = self.parse_optional_generics()?;
        let variants = self.parse_enum_body(&generics)?;
        return Ok(Enum {
            name,
            variants,
            span: start.merge(self.cursor.previous_span()),
        });
    }
    fn parse_enum_type(&mut self, generics: &[String]) -> Result<EnumVariantType, Diagnostic> {
        match self.cursor.expect_token()?.ttype {
            TokenType::LParen => {
                self.cursor.advance();
                match self.cursor.expect_token()?.ttype {
                    TokenType::Enum => {
                        self.cursor.advance();
                        let variants: Vec<EnumVariant> = self.parse_enum_body(generics)?;
                        return Ok(EnumVariantType::Enum(variants));
                    }
                    _ => {
                        return Ok(EnumVariantType::Tuple(
                            self.parse_tuple_type(generics, true)?,
                        ));
                    }
                }
            }
            TokenType::LBrace => {
                let fields = self.parse_kv_struct_body(generics)?;
                return Ok(EnumVariantType::Struct(fields));
            }
            TokenType::Newline | TokenType::RBrace => {
                self.cursor.skip_newline();
                return Ok(EnumVariantType::Regular);
            }
            _ => {
                return Err(Diagnostic {
                    kind: DiagnosticKind::UnexpectedToken {
                        token: self.cursor.current.clone().unwrap(),
                        expected: None,
                    },
                    span: self.cursor.current_span(),
                    info: vec![], // TODO
                });
            }
        }
    }
    fn parse_type_struct(&mut self, generics: &[String]) -> Result<StructFieldType, Diagnostic> {
        match self.cursor.expect_token()?.ttype {
            TokenType::Enum => {
                self.cursor.advance();
                let variants = self.parse_enum_body(generics)?;
                return Ok(StructFieldType::Enum(variants));
            }
            TokenType::LBrace => {
                let fields = self.parse_kv_struct_body(generics)?;
                return Ok(StructFieldType::Struct(fields));
            }
            _ => {
                return Ok(StructFieldType::Regular(self.parse_type(generics)?));
            }
        }
    }
    fn parse_generics(&mut self) -> Result<Vec<String>, Diagnostic> {
        self.cursor.eat(TK::LT)?;
        let mut out = Vec::new();
        if self.cursor.expect_token()?.ttype != TokenType::GT {
            out.push(self.cursor.eat(TK::Identifier)?.as_identifier().clone());
            while self.cursor.expect_token()?.ttype == TokenType::Comma {
                self.cursor.advance();
                if self.cursor.expect_token()?.ttype == TokenType::GT {
                    break;
                }
                out.push(self.cursor.eat(TK::Identifier)?.as_identifier().clone());
            }
        }
        self.cursor.eat(TK::GT)?;
        return Ok(out);
    }
    fn parse_optional_generics(&mut self) -> Result<Vec<String>, Diagnostic> {
        if let Some(t) = &self.cursor.current
            && t.ttype == TokenType::LT
        {
            return Ok(self.parse_generics()?);
        }
        return Ok(Vec::new());
    }
    fn parse_struct(&mut self) -> Result<Struct, Diagnostic> {
        let start = self.cursor.current_span();
        self.cursor.eat(TK::Struct)?;
        let name = self.cursor.eat(TK::Identifier)?.as_identifier().to_owned();
        let generics = self.parse_optional_generics()?;

        match self.cursor.expect_token()?.ttype {
            TokenType::LParen => {
                let fields = self.parse_tuple_type(&generics, false)?;
                return Ok(Struct::Tuple {
                    name,
                    fields,
                    span: start.merge(self.cursor.previous_span()),
                });
            }
            TokenType::LBrace => {
                let fields = self.parse_kv_struct_body(&generics)?;
                return Ok(Struct::KeyValue {
                    fields,
                    name,
                    span: start.merge(self.cursor.previous_span()),
                });
            }
            _ => {
                return Err(Diagnostic {
                    kind: DiagnosticKind::UnexpectedToken {
                        token: self.cursor.current.clone().unwrap(),
                        expected: None,
                    },
                    span: self.cursor.current_span(),
                    info: vec![], // TODO
                });
            }
        }
    }
    fn parse_kv_struct_body(
        &mut self,
        generics: &[String],
    ) -> Result<Vec<StructField>, Diagnostic> {
        self.cursor.eat(TK::LBrace)?;
        self.cursor.skip_newline();
        let name = self.cursor.eat(TK::Identifier)?.as_identifier().to_owned();
        self.cursor.eat(TK::Colon)?;
        let ty = self.parse_type_struct(generics)?;
        let mut fields = vec![StructField { name, ty }];
        self.cursor.skip_newline();
        while self.cursor.expect_token()?.ttype == TokenType::Comma {
            self.cursor.advance();
            self.cursor.skip_newline();
            if self.cursor.expect_token()?.ttype == TokenType::RBrace {
                break;
            }
            let name = self.cursor.eat(TK::Identifier)?.as_identifier().to_owned();
            let ty = self.parse_type_struct(generics)?;
            fields.push(StructField { name, ty })
        }
        self.cursor.eat(TK::RBrace)?;
        return Ok(fields);
    }
    fn item(&mut self) -> Option<Result<StartIRNode, Diagnostic>> {
        self.cursor.skip_newline();
        if let Some(t) = &self.cursor.current {
            match t.ttype {
                TokenType::Func => Some(self.parse_func_body().map(|f| StartIRNode::Function(f))),
                TokenType::Const => Some(self.parse_const().map(|c| StartIRNode::Const(c))),
                TokenType::Import => Some(self.parse_import(false).map(|i| StartIRNode::Import(i))),
                TokenType::Struct => Some(self.parse_struct().map(|s| StartIRNode::Struct(s))),
                TokenType::Enum => Some(self.parse_enum().map(|e| StartIRNode::Enum(e))),
                _ => {
                    self.critical = true;
                    return Some(Err(Diagnostic {
                        kind: DiagnosticKind::UnexpectedToken {
                            token: t.clone(),
                            expected: None,
                        },
                        span: self.cursor.current_span(),
                        info: vec![Info {
                            msg: String::from("expected an item"),
                            span: None,
                            itype: InfoType::Help,
                        }],
                    }));
                }
            }
        } else {
            // return Err(Diagnostic {
            // kind: DiagnosticKind::UnexpectedEOF,
            // span: self.cursor.current_span(),
            // info: vec![],
            // severity: Severity::Err(ErrCode::UnexpectedEOF),
            // });
            return None;
        }
    }
    fn parse_declare(
        &mut self,
        value_required: bool,
    ) -> Result<(String, Option<ASTNode>, Option<TypeAnnotation>), Diagnostic> {
        self.cursor.advance();
        let name = self.cursor.eat(TK::Identifier)?.as_identifier().clone();
        println!("{}", name);
        let mut t = None;
        if let Some(tk) = &self.cursor.current
            && tk.ttype == TokenType::Colon
        {
            self.cursor.advance();
            t = Some(self.parse_type(&[])?)
        }
        let mut value = None;
        if let Some(c) = &self.cursor.current
            && c.ttype == TokenType::Assign
        {
            self.cursor.advance();
            value = Some(self.expr()?);
        } else if value_required {
            self.cursor.advance();
            value = Some(self.expr()?);
        }
        Ok((name, value, t))
    }
    fn parse_const(&mut self) -> Result<Const, Diagnostic> {
        let start_span = self.cursor.current_span();
        let (name, value, ty) = self.parse_declare(true)?;
        Ok(Const {
            span: start_span.merge(self.cursor.previous_span()),
            name,
            value: Box::new(value.unwrap()),
            ty,
        })
    }
    fn parse_if(&mut self) -> Result<If, Diagnostic> {
        let start_span = self.cursor.current_span();
        self.cursor.eat(TK::If)?;
        let condition = self.expr()?;
        self.cursor.skip_newline();
        let body = self.block()?;
        self.cursor.skip_newline();
        let mut else_body = None;
        if let Some(t) = &self.cursor.current
            && t.ttype == TokenType::Else
        {
            self.cursor.advance();
            if self.cursor.expect_token()?.ttype == TokenType::If {
                else_body = Some(IfElseBlock::_ElseIf(self.parse_if()?));
            } else {
                else_body = Some(IfElseBlock::_Block(self.block()?));
            }
        }
        return Ok(If {
            span: start_span.merge(self.cursor.previous_span()),
            condition: Box::new(condition),
            body: Box::new(body),
            else_body: else_body.map(Box::new),
        });
    }
    pub fn statement(&mut self) -> Result<ASTNode, Diagnostic> {
        let start_span = self.cursor.current_span();
        if let Some(t) = &self.cursor.current {
            match t.ttype.clone() {
                TokenType::Let => {
                    let (name, value, t) = self.parse_declare(false)?;
                    return Ok(ASTNode {
                        span: start_span.merge(self.cursor.previous_span()),
                        node: ASTNodeType::Declare(name, value.map(Box::new), t),
                    });
                }
                TokenType::Const => {
                    return self.parse_const().map(|c| ASTNode {
                        span: c.span.clone(),
                        node: ASTNodeType::Const(c),
                    });
                }
                TokenType::Func => {
                    return self.parse_func_body().map(|f| ASTNode {
                        span: f.span.clone(),
                        node: ASTNodeType::Function(f),
                    });
                }
                TokenType::Identifier { name } => {
                    if self.cursor.peek(1).is_some_and(|f| f.ttype == TokenType::Assign) {
                        self.cursor.advance(); // over the name
                        self.cursor.advance(); // over the `=`
                        let val = self.expr()?;
                        return Ok(ASTNode {
                            span: start_span.merge(self.cursor.previous_span()),
                            node: ASTNodeType::Assign(name, Box::new(val)),
                        });
                    } else {
                        return self.expr();
                    }
                }
                TokenType::While => {
                    self.cursor.advance();
                    let condition = Box::new(self.expr()?);
                    let body = Box::new(self.block()?);
                    return Ok(ASTNode {
                        span: start_span.merge(self.cursor.previous_span()),
                        node: ASTNodeType::While { condition, body },
                    });
                }
                TokenType::Return => {
                    self.cursor.advance();
                    let mut expr = None;
                    if let Some(t) = &self.cursor.current
                        && t.ttype != TokenType::Newline
                    {
                        expr = Some(self.expr()?);
                    }
                    return Ok(ASTNode {
                        span: start_span.merge(self.cursor.previous_span()),
                        node: ASTNodeType::Return {
                            expr: expr.map(Box::new),
                        },
                    });
                }
                TokenType::If => {
                    return Ok(ASTNode {
                        node: ASTNodeType::_If(self.parse_if()?),
                        span: start_span.merge(self.cursor.previous_span()),
                    });
                }

                _ => return self.expr(),
            }
        }
        match &self.cursor.current {
            Some(t) => Err(Diagnostic {
                kind: DiagnosticKind::UnexpectedToken {
                    token: t.clone(),
                    expected: None,
                },
                span: start_span.merge(self.cursor.previous_span()),
                info: vec![],
            }),
            None => Err(Diagnostic {
                kind: DiagnosticKind::UnexpectedEOF,
                span: start_span.merge(self.cursor.previous_span()),
                info: vec![],
            }),
        }
    }
    fn unary(&mut self) -> Result<ASTNode, Diagnostic> {
        let start_span = self.cursor.current_span();
        Ok(match &self.cursor.current {
            Some(v) => match v.ttype {
                TokenType::Minus => {
                    self.cursor.advance();
                    let operand = self.unary()?;
                    ASTNode {
                        span: start_span.merge(self.cursor.previous_span()),
                        node: ASTNodeType::UnaryOp(Box::new(operand), UnaryOpType::Negate),
                    }
                }
                TokenType::Plus => {
                    self.cursor.advance();
                    self.unary()?
                }
                _ => self.power()?,
            },
            _ => self.power()?,
        })
    }
    fn equality(&mut self) -> Result<ASTNode, Diagnostic> {
        let mut node = self.comparison()?;
        loop {
            let start_span = self.cursor.current_span();
            if let Some(t) = &self.cursor.current {
                match t.ttype {
                    TokenType::EqualsTo => {
                        self.cursor.advance();
                        let operand = self.add_sub()?;
                        node = ASTNode {
                            span: start_span.merge(self.cursor.previous_span()),
                            node: ASTNodeType::BinOp(
                                Box::new(node),
                                BinOpType::EqualsTo,
                                Box::new(operand),
                            ),
                        }
                    }
                    TokenType::NotEqualsTo => {
                        self.cursor.advance();
                        let operand = self.add_sub()?;
                        node = ASTNode {
                            span: start_span.merge(self.cursor.previous_span()),
                            node: ASTNodeType::BinOp(
                                Box::new(node),
                                BinOpType::NotEqualTo,
                                Box::new(operand),
                            ),
                        }
                    }
                    _ => return Ok(node),
                }
            } else {
                return Ok(node);
            }
        }
    }
    fn or(&mut self) -> Result<ASTNode, Diagnostic> {
        let mut node = self.and()?;
        loop {
            let start_span = self.cursor.current_span();
            if let Some(t) = &self.cursor.current {
                match t.ttype {
                    TokenType::Or => {
                        self.cursor.advance();
                        let operand = self.and()?;
                        node = ASTNode {
                            span: start_span.merge(self.cursor.previous_span()),
                            node: ASTNodeType::BinOp(
                                Box::new(node),
                                BinOpType::Or,
                                Box::new(operand),
                            ),
                        }
                    }
                    _ => return Ok(node),
                }
            } else {
                return Ok(node);
            }
        }
    }

    fn and(&mut self) -> Result<ASTNode, Diagnostic> {
        let mut node = self.not()?;
        loop {
            let start_span = self.cursor.current_span();
            if let Some(t) = &self.cursor.current {
                match t.ttype {
                    TokenType::And => {
                        self.cursor.advance();
                        let operand = self.not()?;
                        node = ASTNode {
                            span: start_span.merge(self.cursor.previous_span()),
                            node: ASTNodeType::BinOp(
                                Box::new(node),
                                BinOpType::And,
                                Box::new(operand),
                            ),
                        }
                    }
                    _ => return Ok(node),
                }
            } else {
                return Ok(node);
            }
        }
    }

    fn not(&mut self) -> Result<ASTNode, Diagnostic> {
        let start_span = self.cursor.current_span();
        if let Some(t) = &self.cursor.current {
            if t.ttype == TokenType::Not {
                self.cursor.advance();
                let operand = self.not()?; // not self.equality()!
                return Ok(ASTNode {
                    span: start_span.merge(self.cursor.previous_span()),
                    node: ASTNodeType::UnaryOp(Box::new(operand), UnaryOpType::Not),
                });
            }
        }
        self.equality()
    }
    fn comparison(&mut self) -> Result<ASTNode, Diagnostic> {
        let mut node = self.add_sub()?;
        loop {
            let start_span = self.cursor.current_span();
            if let Some(t) = &self.cursor.current {
                match t.ttype {
                    TokenType::LT => {
                        self.cursor.advance();
                        let operand = self.add_sub()?;
                        node = ASTNode {
                            span: start_span.merge(self.cursor.previous_span()),
                            node: ASTNodeType::BinOp(
                                Box::new(node),
                                BinOpType::LT,
                                Box::new(operand),
                            ),
                        }
                    }
                    TokenType::GT => {
                        self.cursor.advance();
                        let operand = self.add_sub()?;
                        node = ASTNode {
                            span: start_span.merge(self.cursor.previous_span()),
                            node: ASTNodeType::BinOp(
                                Box::new(node),
                                BinOpType::GT,
                                Box::new(operand),
                            ),
                        }
                    }
                    TokenType::LTE => {
                        self.cursor.advance();
                        let operand = self.add_sub()?;
                        node = ASTNode {
                            span: start_span.merge(self.cursor.previous_span()),
                            node: ASTNodeType::BinOp(
                                Box::new(node),
                                BinOpType::LTE,
                                Box::new(operand),
                            ),
                        }
                    }
                    TokenType::GTE => {
                        self.cursor.advance();
                        let operand = self.add_sub()?;
                        node = ASTNode {
                            span: start_span.merge(self.cursor.previous_span()),
                            node: ASTNodeType::BinOp(
                                Box::new(node),
                                BinOpType::GTE,
                                Box::new(operand),
                            ),
                        }
                    }
                    _ => return Ok(node),
                }
            } else {
                return Ok(node);
            }
        }
    }
    fn add_sub(&mut self) -> Result<ASTNode, Diagnostic> {
        let mut node = self.mul_div()?;
        loop {
            let start_span = self.cursor.current_span();
            if let Some(t) = &self.cursor.current {
                match t.ttype {
                    TokenType::Minus => {
                        self.cursor.advance();
                        let operand = self.mul_div()?;
                        node = ASTNode {
                            span: start_span.merge(self.cursor.previous_span()),
                            node: ASTNodeType::BinOp(
                                Box::new(node),
                                BinOpType::Subtract,
                                Box::new(operand),
                            ),
                        }
                    }
                    TokenType::Plus => {
                        self.cursor.advance();
                        let operand = self.mul_div()?;
                        node = ASTNode {
                            span: start_span.merge(self.cursor.previous_span()),
                            node: ASTNodeType::BinOp(
                                Box::new(node),
                                BinOpType::Add,
                                Box::new(operand),
                            ),
                        }
                    }
                    _ => return Ok(node),
                }
            } else {
                return Ok(node);
            }
        }
    }
    fn mul_div(&mut self) -> Result<ASTNode, Diagnostic> {
        let mut node = self.unary()?;
        loop {
            let start_span = self.cursor.current_span();
            if let Some(t) = &self.cursor.current {
                match t.ttype {
                    TokenType::Multiply => {
                        self.cursor.advance();
                        let operand = self.unary()?;
                        node = ASTNode {
                            span: start_span.merge(self.cursor.previous_span()),
                            node: ASTNodeType::BinOp(
                                Box::new(node),
                                BinOpType::Multiply,
                                Box::new(operand),
                            ),
                        };
                    }
                    TokenType::Divide => {
                        self.cursor.advance();
                        let operand = self.unary()?;
                        node = ASTNode {
                            span: start_span.merge(self.cursor.previous_span()),
                            node: ASTNodeType::BinOp(
                                Box::new(node),
                                BinOpType::Divide,
                                Box::new(operand),
                            ),
                        };
                    }
                    TokenType::Mod => {
                        self.cursor.advance();
                        let operand = self.unary()?;
                        node = ASTNode {
                            span: start_span.merge(self.cursor.previous_span()),
                            node: ASTNodeType::BinOp(
                                Box::new(node),
                                BinOpType::Mod,
                                Box::new(operand),
                            ),
                        }
                    }
                    _ => return Ok(node),
                }
            } else {
                return Ok(node);
            }
        }
    }
    fn power(&mut self) -> Result<ASTNode, Diagnostic> {
        let start_span = self.cursor.current_span();
        let node = self.postfix()?;
        Ok(match &self.cursor.current {
            Some(v) => {
                if v.ttype == TokenType::Power {
                    self.cursor.advance();
                    let p = self.unary()?;
                    ASTNode {
                        span: start_span.merge(self.cursor.previous_span()),
                        node: ASTNodeType::BinOp(Box::new(node), BinOpType::Power, Box::new(p)),
                    }
                } else {
                    node
                }
            }
            _ => node,
        })
    }
    fn parse_import(&mut self, _sub_import: bool) -> Result<Import, Diagnostic> {
        // let start = self.cursor.current_span();
        // if !sub_import {
        //     self.cursor.eat(TK::Import);
        // }
        // let path = self.path()?;
        // let mut children = Vec::new();
        // let alias = None;
        // if let Some(t) = &self.cursor.current
        //     && t.ttype == TokenType::As
        // {
        //     self.cursor.advance();
        // }
        // if let Some(t) = &self.cursor.current
        //     && t.ttype == TokenType::LBrace
        // {
        //     self.cursor.advance();
        //     children.push(self.parse_import(true)?);
        //     while self.cursor.expect_token()?.ttype == TokenType::Comma {
        //         self.cursor.advance();
        //         self.skip_newline();
        //         if self.cursor.expect_token()?.ttype == TokenType::RBrace {
        //             break;
        //         }
        //         children.push(self.parse_import(true)?);
        //     }
        // }
        todo!()
    }
    fn postfix(&mut self) -> Result<ASTNode, Diagnostic> {
        let mut expr = self.primary()?;
        loop {
            match &self.cursor.current {
                Some(t) => match t.ttype {
                    TokenType::Dot => {
                        self.cursor.advance();
                        let token = self.cursor.eat_info(
                            TK::Identifier,
                            vec![Info {
                                itype: InfoType::Note,
                                msg: "attributes must be identifiers like `.foo`".to_owned(),
                                span: None,
                            }],
                        )?;
                        let name = token.as_identifier();
                        expr = ASTNode {
                            node: ASTNodeType::Attribute(Box::new(expr.clone()), name.to_owned()),
                            span: expr.span.merge(self.cursor.current_span()),
                        }
                    }
                    TokenType::LBracket => {
                        self.cursor.advance();
                        let idx = self.expr()?;
                        expr = ASTNode {
                            span: expr.span.merge(self.cursor.current_span()),
                            node: ASTNodeType::Index(Box::new(expr), Box::new(idx)),
                        }
                    }
                    TokenType::LParen => {
                        self.cursor.advance();
                        let mut args = vec![];
                        if self.cursor.expect_token()?.ttype != TokenType::RParen {
                            args.push(self.expr()?);
                        }
                        while self.cursor.expect_token()?.ttype == TokenType::Comma {
                            self.cursor.eat(TK::Comma)?;
                            if self.cursor.expect_token()?.ttype != TokenType::RParen {
                                args.push(self.expr()?);
                            }
                        }
                        self.cursor.eat(TK::RParen)?;
                        expr = ASTNode {
                            span: expr.span.merge(self.cursor.current_span()),
                            node: ASTNodeType::Call(Box::new(expr), args),
                        }
                    }
                    _ => break,
                },
                _ => break,
            }
        }
        Ok(expr)
    }
    pub fn primary(&mut self) -> Result<ASTNode, Diagnostic> {
        let start_span = self.cursor.current_span();
        match self.cursor.current.clone() {
            Some(t) => match &t.ttype {
                TokenType::Identifier { .. } => self.parse_path_or_ident(),

                TokenType::String { val } => {
                    self.cursor.advance();
                    Ok(ASTNode {
                        span: start_span,
                        node: ASTNodeType::String(val.to_owned()),
                    })
                }
                TokenType::Float { val } => {
                    self.cursor.advance();
                    Ok(ASTNode {
                        span: start_span,
                        node: ASTNodeType::Float(*val),
                    })
                }
                TokenType::Int { val } => {
                    self.cursor.advance();
                    Ok(ASTNode {
                        span: start_span,
                        node: ASTNodeType::Int(*val),
                    })
                }
                TokenType::True => {
                    self.cursor.advance();
                    Ok(ASTNode {
                        span: start_span,
                        node: ASTNodeType::Bool(true),
                    })
                }
                TokenType::False => {
                    self.cursor.advance();
                    Ok(ASTNode {
                        span: start_span,
                        node: ASTNodeType::Bool(false),
                    })
                }

                TokenType::LParen => {
                    self.cursor.advance();
                    if self.cursor.expect_token()?.ttype == TokenType::RParen {
                        self.cursor.advance();
                        Ok(ASTNode {
                            span: start_span,
                            node: ASTNodeType::Unit,
                        })
                    } else {
                        let node = self.expr()?;
                        self.cursor.eat(TK::RParen)?;
                        Ok(node)
                    }
                }
                _ => {
                    self.cursor.advance();
                    Err(Diagnostic {
                        kind: DiagnosticKind::UnexpectedToken {
                            token: t.clone(),
                            expected: None,
                        },
                        info: vec![],
                        span: start_span,
                    })
                }
            },
            None => Err(Diagnostic {
                kind: DiagnosticKind::UnexpectedEOF,
                span: start_span,
                info: vec![],
            }),
        }
    }
}

struct FunctionArg {
    name: String,
    t: TypeAnnotation,
}

#[derive(Debug)]
pub struct PlatformFileDeclaration {
    version: i32,
    item: PlatformFileDeclarationItem,
    idx: i32,
    span: Span,
}
#[derive(Debug)]
enum PlatformFileDeclarationItem {
    Function {
        name: String,
        params: Vec<FunctionParameter>,
        ret: TypeAnnotation,
    },
}

pub struct PlatformFile {
    version: i32,
    declarations: Vec<PlatformFileDeclaration>,
}

impl Parser {
    pub fn parse_plt_file_stmt(&mut self) -> Result<Option<PlatformFileDeclaration>, Diagnostic> {
        let mut decl: Option<PlatformFileDeclarationItem> = None;
        self.cursor.skip_newline();
        let start = self.cursor.current_span();
        if let Some(t) = &self.cursor.current {
            match t.ttype {
                TokenType::Func => {
                    let (_, name, params, ret, ..) = self.parse_func(true)?;
                    decl = Some(PlatformFileDeclarationItem::Function { name, params, ret });
                }
                TokenType::AtRate => {
                    self.cursor.advance();
                    match self.cursor.expect_token()?.ttype {
                        TokenType::Version => {
                            self.cursor.advance();
                            let ver = match &self.cursor.current {
                                Some(t) => match t.ttype {
                                    TokenType::Int { val } => {
                                        self.cursor.advance();
                                        val
                                    }
                                    _ => self.version + 1,
                                },
                                None => self.version + 1,
                            };
                            self.version = ver;
                        }
                        _ => todo!(),
                    }
                }
                _ => {
                    dbg!(&self.cursor.current);
                    todo!()
                }
            }
            if let Some(item) = decl {
                let idx = match &self.cursor.current {
                    None => self.indexes.last().expect("(internal) no indexes left") + 1,
                    Some(t) => match t.ttype {
                        TokenType::Int { val } => val,
                        _ => self.indexes.last().expect("(internal) no indexes left") + 1,
                    },
                };
                if self.indexes.contains(&idx) {
                    return Err(Diagnostic {
                        kind: DiagnosticKind::DuplicateIndex { idx },
                        span: self.cursor.current_span(),
                        info: Vec::new(),
                    });
                }
                self.indexes.push(idx);
                self.cursor.advance();
                return Ok(Some(PlatformFileDeclaration {
                    version: self.version,
                    item,
                    idx,
                    span: start.merge(self.cursor.previous_span()),
                }));
            }
            return Ok(None);
        }
        todo!()
    }
}

/*

// func println(item: str) -> () 13
            ^
*/
