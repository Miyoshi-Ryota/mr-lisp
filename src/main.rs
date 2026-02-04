use mr_lisp::eval::*;
use std::cell::RefCell;
use std::rc::Rc;

use linefeed::{Interface, ReadResult};
use mr_lisp::parser::Object;

const PROMPT: &str = "mr-lisp> ";
const CONTINUATION_PROMPT: &str = "....> ";

fn update_paren_balance(line: &str, balance: &mut i32, in_string: &mut bool) {
    for ch in line.chars() {
        match ch {
            '"' => {
                *in_string = !*in_string;
            }
            '(' if !*in_string => {
                *balance += 1;
            }
            ')' if !*in_string => {
                *balance -= 1;
            }
            _ => {}
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let reader = Interface::new(PROMPT).unwrap();
    let mut env = Rc::new(RefCell::new(Env::new()));
    let mut buffer = String::new();
    let mut paren_balance: i32 = 0;
    let mut in_string = false;

    reader.set_prompt(format!("{}", PROMPT).as_ref()).unwrap();

    while let ReadResult::Input(input) = reader.read_line().unwrap() {
        if buffer.is_empty() && input.eq("exit") {
            break;
        }

        update_paren_balance(&input, &mut paren_balance, &mut in_string);
        if !buffer.is_empty() {
            buffer.push('\n');
        }
        buffer.push_str(&input);

        if in_string || paren_balance > 0 {
            reader.set_prompt(format!("{}", CONTINUATION_PROMPT).as_ref()).unwrap();
            continue;
        }

        if paren_balance < 0 {
            eprintln!("ParseError: Unexpected ')'");
            buffer.clear();
            paren_balance = 0;
            in_string = false;
            reader.set_prompt(format!("{}", PROMPT).as_ref()).unwrap();
            continue;
        }

        let program = buffer.trim();
        if program.is_empty() {
            buffer.clear();
            paren_balance = 0;
            in_string = false;
            reader.set_prompt(format!("{}", PROMPT).as_ref()).unwrap();
            continue;
        }

        let val = eval(program, &mut env)?;
        match val {
            Object::Void => {}
            Object::Integer(n) => println!("{}", n),
            Object::Bool(b) => println!("{}", b),
            Object::Symbol(s) => println!("{}", s),
            Object::Lambda(params, body) => {
                println!("Lambda(");
                for param in params {
                    println!("{} ", param);
                }
                println!(")");
                for expr in (*body).iter() {
                    println!(" {}", expr);
                }
            }
            _ => println!("{}", val),
        }

        buffer.clear();
        paren_balance = 0;
        in_string = false;
        reader.set_prompt(format!("{}", PROMPT).as_ref()).unwrap();
    }

    println!("Good bye");
    Ok(())
}
