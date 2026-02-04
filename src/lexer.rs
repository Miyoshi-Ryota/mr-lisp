use std::{collections::HashSet, str::Chars};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Integer(i64),
    Symbol(String),
    LParen,
    RParen,
    Float(f64),
    String(String),
    BinaryOp(String), //  今後、　enum にするかも
    Keyword(String),
}

struct Tokenizer<'a> {
    input: Chars<'a>,
    current_char: Option<char>,
    keywords: HashSet<&'a str>,
    binary_ops: HashSet<char>,
}

impl<'a> Tokenizer<'a> {
    fn new(input: &'a str) -> Self {
        let mut chars = input.chars();
        let current_char = chars.next();
        let tokenizer = Tokenizer {
            input: chars,
            current_char: current_char,
            keywords: [
                "define", "list", "print", "lambda", "range", "cons", "car", "cdr", "length",
                "null?", "begin", "let", "if", "else", "cond",
            ]
            .into_iter()
            .collect(),
            binary_ops: ['+', '-', '*', '/', '%', '<', '>', '=', '|', '&']
                .into_iter()
                .collect(),
        };
        tokenizer
    }

    fn advance(&mut self) -> Option<char> {
        self.current_char = self.input.next();
        self.current_char
    }

    fn eat_whitespace(&mut self) {
        while let Some(c) = self.current_char {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_symbol(&mut self) -> String {
        let mut symbol = String::new();
        while let Some(c) = self.current_char {
            if !c.is_whitespace() && c != '(' && c != ')' {
                symbol.push(c);
                self.advance();
            } else {
                break;
            }
        }
        symbol
    }

    fn read_number(&mut self) -> String {
        let mut number = String::new();
        while let Some(c) = self.current_char {
            if c.is_digit(10) || c == '.' {
                number.push(c);
                self.advance();
            } else {
                break;
            }
        }
        number
    }

    fn read_string(&mut self) -> String {
        let mut string = String::new();
        self.advance(); // Skip the opening quote
        while let Some(c) = self.current_char {
            if c != '"' {
                string.push(c);
                self.advance();
            } else {
                break;
            }
        }
        self.advance(); // Skip the closing quote
        string
    }

    fn next_token(&mut self) -> Option<Token> {
        self.eat_whitespace();
        match self.current_char? {
            '(' => {
                self.advance();
                Some(Token::LParen)
            }
            ')' => {
                self.advance();
                Some(Token::RParen)
            }
            '"' => {
                let string = self.read_string();
                Some(Token::String(string))
            }
            c if c.is_digit(10) => {
                let number_str = self.read_number();
                if number_str.contains('.') {
                    Some(Token::Float(number_str.parse().unwrap()))
                } else {
                    Some(Token::Integer(number_str.parse().unwrap()))
                }
            }
            c if self.binary_ops.contains(&c) => {
                let op = c.to_string();
                self.advance();
                Some(Token::BinaryOp(op))
            }
            c if c.is_alphabetic() || c == '_' => {
                let symbol = self.read_symbol();
                if self.keywords.contains(symbol.as_str()) {
                    Some(Token::Keyword(symbol))
                } else {
                    Some(Token::Symbol(symbol))
                }
            }
            _ => None,
        }
    }
}

pub fn tokenize(input: &str) -> Vec<Token> {
    // Result型にするべきかも。今不正な入力をした時にどうなるか不明。
    let mut tokenizer = Tokenizer::new(input);
    let mut tokens = Vec::new();
    while let Some(token) = tokenizer.next_token() {
        tokens.push(token);
    }
    tokens
}

#[cfg(test)]
mod tests {
    use crate::lexer::{Token, tokenize};

    #[test]
    fn test_tokenize() {
        let input = "(define sqr (* x x))";
        let tokens = vec![
            Token::LParen,
            Token::Keyword("define".to_string()),
            Token::Symbol("sqr".to_string()),
            Token::LParen,
            Token::BinaryOp("*".to_string()),
            Token::Symbol("x".to_string()),
            Token::Symbol("x".to_string()),
            Token::RParen,
            Token::RParen,
        ];
        assert_eq!(tokenize(input), tokens);
    }

    #[test]
    fn test_area_of_a_circle() {
        let program = "
            (
                (define r 10)
                (define pi 314)
                (* pi (* r r))
            )
        ";
        let tokens = tokenize(program);
        assert_eq!(
            tokens,
            vec![
                Token::LParen,
                Token::LParen,
                Token::Keyword("define".to_string()),
                Token::Symbol("r".to_string()),
                Token::Integer(10),
                Token::RParen,
                Token::LParen,
                Token::Keyword("define".to_string()),
                Token::Symbol("pi".to_string()),
                Token::Integer(314),
                Token::RParen,
                Token::LParen,
                Token::BinaryOp("*".to_string()),
                Token::Symbol("pi".to_string()),
                Token::LParen,
                Token::BinaryOp("*".to_string()),
                Token::Symbol("r".to_string()),
                Token::Symbol("r".to_string()),
                Token::RParen,
                Token::RParen,
                Token::RParen
            ]
        );
    }
}
