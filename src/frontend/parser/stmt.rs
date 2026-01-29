// src/frontend/parser/stmt.rs
use crate::frontend::ast::AstNode;
use nom::IResult;
use nom::Parser;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::combinator::opt;
use nom::multi::many0;
use nom::sequence::{delimited, preceded};

use super::expr::parse_full_expr;
use super::parser::{parse_ident, ws};

pub fn parse_assign(input: &str) -> IResult<&str, AstNode> {
    // let/mut bindings (types are accepted but ignored by the parser)
    if let Ok((input, (_let_kw, _mut_kw, name, _ty_opt))) = (
        opt(ws(tag("let"))),
        opt(ws(tag("mut"))),
        ws(parse_ident),
        opt(preceded(ws(tag(":")), ws(parse_ident))),
    )
        .parse(input)
    {
        let (input, op) = ws(alt((tag("+="), tag("-="), tag("*="), tag("/="), tag("=")))).parse(input)?;
        let (input, rhs) = ws(parse_full_expr).parse(input)?;
        let lhs = AstNode::Var(name);
        let rhs = match op {
            "+=" => AstNode::BinaryOp {
                op: "+".to_string(),
                left: Box::new(lhs.clone()),
                right: Box::new(rhs),
            },
            "-=" => AstNode::BinaryOp {
                op: "-".to_string(),
                left: Box::new(lhs.clone()),
                right: Box::new(rhs),
            },
            "*=" => AstNode::BinaryOp {
                op: "*".to_string(),
                left: Box::new(lhs.clone()),
                right: Box::new(rhs),
            },
            "/=" => AstNode::BinaryOp {
                op: "/".to_string(),
                left: Box::new(lhs.clone()),
                right: Box::new(rhs),
            },
            _ => rhs,
        };
        return Ok((input, AstNode::Assign(Box::new(lhs), Box::new(rhs))));
    }

    let (input, lhs) = ws(parse_full_expr).parse(input)?;
    let (input, op) = ws(alt((tag("+="), tag("-="), tag("*="), tag("/="), tag("=")))).parse(input)?;
    let (input, rhs) = ws(parse_full_expr).parse(input)?;
    let rhs = match op {
        "+=" => AstNode::BinaryOp {
            op: "+".to_string(),
            left: Box::new(lhs.clone()),
            right: Box::new(rhs),
        },
        "-=" => AstNode::BinaryOp {
            op: "-".to_string(),
            left: Box::new(lhs.clone()),
            right: Box::new(rhs),
        },
        "*=" => AstNode::BinaryOp {
            op: "*".to_string(),
            left: Box::new(lhs.clone()),
            right: Box::new(rhs),
        },
        "/=" => AstNode::BinaryOp {
            op: "/".to_string(),
            left: Box::new(lhs.clone()),
            right: Box::new(rhs),
        },
        _ => rhs,
    };
    Ok((input, AstNode::Assign(Box::new(lhs), Box::new(rhs))))
}

pub fn parse_return(input: &str) -> IResult<&str, AstNode> {
    let (input, _) = ws(tag("return")).parse(input)?;
    let (input, inner) = ws(parse_full_expr).parse(input)?;
    Ok((input, AstNode::Return(Box::new(inner))))
}

pub fn parse_if(input: &str) -> IResult<&str, AstNode> {
    let (input, _) = ws(tag("if")).parse(input)?;
    let (input, cond) = ws(parse_full_expr).parse(input)?;
    let (input, then) =
        delimited(ws(tag("{")), many0(ws(parse_stmt)), ws(tag("}"))).parse(input)?;
    let (input, else_opt) = opt(preceded(
        ws(tag("else")),
        delimited(ws(tag("{")), many0(ws(parse_stmt)), ws(tag("}"))),
    ))
    .parse(input)?;
    let else_ = else_opt.unwrap_or_default();
    Ok((
        input,
        AstNode::If {
            cond: Box::new(cond),
            then,
            else_,
        },
    ))
}

pub fn parse_stmt(input: &str) -> IResult<&str, AstNode> {
    let (input, stmt) = alt((
        parse_assign,
        parse_return,
        parse_if,
        // Expression statement (e.g. function call with side-effects)
        parse_full_expr.map(|expr| AstNode::ExprStmt {
            expr: Box::new(expr),
        }),
    ))
    .parse(input)?;
    let (input, _) = opt(ws(tag(";"))).parse(input)?;
    Ok((input, stmt))
}
