use std::{error::Error, fmt, rc::Rc};

use crate::lexer::{Token, tokenize};

#[derive(Debug, Clone, PartialEq)]
pub enum Object {
    Void,
    Keyword(String),
    BinaryOp(String),
    Integer(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Symbol(String),
    ListData(Vec<Object>), // 評価後のListというか、データというか、cdrとかの引数になるListのようなイメージ。
    Lambda(Vec<String>, Vec<Object>),
    List(Rc<Vec<Object>>), // S式というかASTというかプログラムを表すList。
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Object::Void => write!(f, "Void"),
            Object::Keyword(s) => write!(f, "{}", s),
            Object::BinaryOp(s) => write!(f, "{}", s),
            Object::Integer(i) => write!(f, "{}", i),
            Object::Float(fl) => write!(f, "{}", fl),
            Object::Bool(b) => write!(f, "{}", b),
            Object::String(s) => write!(f, "{}", s),
            Object::Symbol(s) => write!(f, "{}", s),
            Object::Lambda(params, body) => {
                let params_str = params.join(" ");
                let body_str: Vec<String> = body.iter().map(|obj| format!("{}", obj)).collect();
                write!(f, "Lambda({}) {}", params_str, body_str.join(" "))
            }
            Object::List(list) => {
                let elements: Vec<String> = list.iter().map(|obj| format!("{}", obj)).collect();
                write!(f, "({})", elements.join(" "))
            }
            Object::ListData(list) => {
                let elements: Vec<String> = list.iter().map(|obj| format!("{}", obj)).collect();
                write!(f, "({})", elements.join(" "))
            }
        }
    }
}

#[derive(Debug)]
pub struct ParseError {
    message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ParseError: {}", self.message)
    }
}

impl Error for ParseError {}

pub fn parse(program: &str) -> Result<Object, ParseError> {
    let mut tokens = tokenize(program);
    tokens.reverse(); // トークンを逆順にしてスタックのように扱う
    let parsed_list = parse_list(&mut tokens)?;
    Ok(parsed_list)
}

fn parse_list(tokens: &mut Vec<Token>) -> Result<Object, ParseError> {
    let token = tokens.pop();
    if token != Some(Token::LParen) {
        return Err(ParseError {
            message: "Expected '(' at the beginning of list".to_string(),
        });
    }
    let mut list: Vec<Object> = Vec::new();
    while !tokens.is_empty() {
        let token = tokens.pop();
        if token.is_none() {
            return Err(ParseError {
                message: "Unexpected end of input while parsing list".to_string(),
            });
        }

        let t = token.unwrap();
        match t {
            Token::Integer(i) => list.push(Object::Integer(i)),
            Token::Float(f) => list.push(Object::Float(f)),
            Token::String(s) => list.push(Object::String(s)),
            Token::Symbol(s) => list.push(Object::Symbol(s)),
            Token::LParen => {
                tokens.push(Token::LParen);
                let sublist = parse_list(tokens)?;
                list.push(sublist);
            }
            Token::RParen => {
                return Ok(Object::List(Rc::new(list)));
            }
            Token::BinaryOp(op) => list.push(Object::BinaryOp(op)),
            Token::Keyword(kw) => list.push(Object::Keyword(kw)),
        }
    }
    Err(ParseError {
        message: "Expected ')' at the end of list".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let list = parse("(+ 1 2)").unwrap();
        assert_eq!(
            list,
            Object::List(Rc::new(vec![
                Object::BinaryOp("+".to_string()),
                Object::Integer(1),
                Object::Integer(2),
            ]))
        );
    }

    #[test]
    fn test_area_of_a_circle() {
        let program = "(
                         (define r 10)
                         (define pi 314)
                         (* pi (* r r))
                       )";
        let list = parse(program).unwrap();
        assert_eq!(
            list,
            Object::List(Rc::new(vec![
                Object::List(Rc::new(vec![
                    Object::Keyword("define".to_string()),
                    Object::Symbol("r".to_string()),
                    Object::Integer(10),
                ])),
                Object::List(Rc::new(vec![
                    Object::Keyword("define".to_string()),
                    Object::Symbol("pi".to_string()),
                    Object::Integer(314),
                ])),
                Object::List(Rc::new(vec![
                    Object::BinaryOp("*".to_string()),
                    Object::Symbol("pi".to_string()),
                    Object::List(Rc::new(vec![
                        Object::BinaryOp("*".to_string()),
                        Object::Symbol("r".to_string()),
                        Object::Symbol("r".to_string()),
                    ])),
                ])),
            ]))
        );
    }
}
