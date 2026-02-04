use crate::parser::Object;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub fn eval(program: &str, env: &mut Rc<RefCell<Env>>) -> Result<Object, String> {
    let ast = crate::parser::parse(program).map_err(|e| e.to_string())?;
    eval_obj(&ast, env)
}

fn eval_obj(obj: &Object, env: &mut Rc<RefCell<Env>>) -> Result<Object, String> {
    match obj {
        Object::Void => Ok(Object::Void),
        Object::Bool(b) => Ok(Object::Bool(*b)),
        Object::Integer(n) => Ok(Object::Integer(*n)),
        Object::Float(f) => Ok(Object::Float(*f)),
        Object::ListData(list) => eval_list_data(list, env),
        Object::String(s) => Ok(Object::String(s.clone())),
        Object::Symbol(s) => eval_symbol(s, env),
        Object::Lambda(_, _) => Ok(Object::Void), // 仮
        Object::List(list) => eval_list(list, env),
        _ => Err(format!("Invalid object: {:?}", obj)),
    }
}

pub struct Env {
    parent: Option<Rc<RefCell<Env>>>,
    vars: HashMap<String, Object>,
}

impl Env {
    pub fn new() -> Self {
        Env {
            parent: None,
            vars: HashMap::new(),
        }
    }

    pub fn update(&mut self, data: Rc<RefCell<Self>>) {
        self.vars.extend(
            data.borrow()
                .vars
                .iter()
                .map(|(k, v)| (k.clone(), v.clone())),
        )
    }

    pub fn extend(parent: Rc<RefCell<Self>>) -> Self {
        Env {
            parent: Some(parent),
            vars: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<Object> {
        match self.vars.get(name) {
            Some(value) => Some(value.clone()),
            None => self
                .parent
                .as_ref()
                .and_then(|o| o.borrow().get(name).clone()),
        }
    }

    pub fn set(&mut self, name: &str, val: Object) {
        self.vars.insert(name.to_string(), val);
    }
}

fn eval_list_data(_list: &Vec<Object>, _env: &mut Rc<RefCell<Env>>) -> Result<Object, String> {
    unimplemented!();
}

fn eval_symbol(symbol: &String, env: &Rc<RefCell<Env>>) -> Result<Object, String> {
    match env.borrow().get(symbol.as_str()) {
        Some(value) => Ok(value),
        None => Err(format!("Undefined symbol: {}", symbol)),
    }
}

fn eval_list(list: &Rc<Vec<Object>>, env: &mut Rc<RefCell<Env>>) -> Result<Object, String> {
    let head = list.first().ok_or("Empty list")?;
    match head {
        Object::Keyword(_) => eval_keyword(list, env),
        Object::BinaryOp(_) => eval_binary_op(list, env),
        Object::Symbol(s) => eval_function_call(s, list, env),
        _ => Err(format!("Invalid list op: {:?}", list)),
    }
}

fn eval_keyword(list: &Rc<Vec<Object>>, env: &mut Rc<RefCell<Env>>) -> Result<Object, String> {
    if list.is_empty() {
        return Err("Empty keyword list".to_string());
    }
    let keyword = match &list[0] {
        Object::Keyword(kw) => kw.as_str(),
        _ => return Err(format!("Expected keyword, found {:?}", list[0])),
    };
    match keyword {
        "begin" => eval_begin(list, env),
        "define" => eval_define(list, env),
        "if" => eval_if(list, env),
        "lambda" => eval_function_definition(list, env),
        _ => Err(format!("Unsupported keyword: {}", keyword)),
    }
}

fn eval_begin(list: &Vec<Object>, env: &mut Rc<RefCell<Env>>) -> Result<Object, String> {
    let mut result = Object::Void;
    for expr in &list[1..] {
        result = eval_obj(expr, env)?;
    }
    Ok(result)
}

fn eval_define(list: &Vec<Object>, env: &mut Rc<RefCell<Env>>) -> Result<Object, String> {
    let sym = match &list[1] {
        Object::Symbol(s) => s.clone(),
        _ => return Err(format!("Invalid define syntax: {:?}", list)),
    };

    let val = eval_obj(&list[2], env)?;
    env.borrow_mut().set(&sym, val);
    Ok(Object::Void)
}

fn eval_binary_op(list: &[Object], env: &mut Rc<RefCell<Env>>) -> Result<Object, String> {
    if list.len() != 3 {
        return Err(format!("Invalid binary operation: {:?}", list));
    }

    let op = list[0].clone();
    let left = eval_obj(&list[1], env)?;
    let right = eval_obj(&list[2], env)?;

    match op {
        Object::BinaryOp(s) => match s.as_str() {
            "+" => match (left, right) {
                (Object::Integer(l), Object::Integer(r)) => Ok(Object::Integer(l + r)),
                (Object::Float(l), Object::Float(r)) => Ok(Object::Float(l + r)),
                (Object::Integer(l), Object::Float(r)) => Ok(Object::Float(l as f64 + r)),
                (Object::Float(l), Object::Integer(r)) => Ok(Object::Float(l + r as f64)),
                (left, right) => Err(format!("Invalid operands for +: {:?}, {:?}", &left, right)),
            },
            "-" => match (left, right) {
                (Object::Integer(l), Object::Integer(r)) => Ok(Object::Integer(l - r)),
                (Object::Float(l), Object::Float(r)) => Ok(Object::Float(l - r)),
                (Object::Integer(l), Object::Float(r)) => Ok(Object::Float(l as f64 - r)),
                (Object::Float(l), Object::Integer(r)) => Ok(Object::Float(l - r as f64)),
                (left, right) => Err(format!("Invalid operands for -: {:?}, {:?}", left, right)),
            },
            "*" => match (left, right) {
                (Object::Integer(l), Object::Integer(r)) => Ok(Object::Integer(l * r)),
                (Object::Float(l), Object::Float(r)) => Ok(Object::Float(l * r)),
                (Object::Integer(l), Object::Float(r)) => Ok(Object::Float(l as f64 * r)),
                (Object::Float(l), Object::Integer(r)) => Ok(Object::Float(l * r as f64)),
                (left, right) => Err(format!("Invalid operands for *: {:?}, {:?}", left, right)),
            },
            "/" => match (left, right) {
                (Object::Integer(l), Object::Integer(r)) => {
                    if r == 0 {
                        Err("Division by zero".to_string())
                    } else {
                        Ok(Object::Integer(l / r))
                    }
                }
                (Object::Float(l), Object::Float(r)) => {
                    if r == 0.0 {
                        Err("Division by zero".to_string())
                    } else {
                        Ok(Object::Float(l / r))
                    }
                }
                (Object::Integer(l), Object::Float(r)) => {
                    if r == 0.0 {
                        Err("Division by zero".to_string())
                    } else {
                        Ok(Object::Float(l as f64 / r))
                    }
                }
                (Object::Float(l), Object::Integer(r)) => {
                    if r == 0 {
                        Err("Division by zero".to_string())
                    } else {
                        Ok(Object::Float(l / r as f64))
                    }
                }
                (left, right) => Err(format!("Invalid operands for /: {:?}, {:?}", left, right)),
            },
            "<" => match (left, right) {
                (Object::Integer(l), Object::Integer(r)) => Ok(Object::Bool(l < r)),
                (Object::Float(l), Object::Float(r)) => Ok(Object::Bool(l < r)),
                (Object::Integer(l), Object::Float(r)) => Ok(Object::Bool((l as f64) < r)),
                (Object::Float(l), Object::Integer(r)) => Ok(Object::Bool(l < (r as f64))),
                (left, right) => Err(format!("Invalid operands for <: {:?}, {:?}", left, right)),
            },
            ">" => match (left, right) {
                (Object::Integer(l), Object::Integer(r)) => Ok(Object::Bool(l > r)),
                (Object::Float(l), Object::Float(r)) => Ok(Object::Bool(l > r)),
                (Object::Integer(l), Object::Float(r)) => Ok(Object::Bool((l as f64) > r)),
                (Object::Float(l), Object::Integer(r)) => Ok(Object::Bool(l > (r as f64))),
                (left, right) => Err(format!("Invalid operands for >: {:?}, {:?}", left, right)),
            },
            _ => Err(format!("Unsupported binary operator: {}", s)),
        },
        _ => Err(format!("Invalid binary operation: {:?}", op)),
    }
}

fn eval_if(list: &Vec<Object>, env: &mut Rc<RefCell<Env>>) -> Result<Object, String> {
    let cond_obj = eval_obj(&list[1], env)?;
    let cond = match cond_obj {
        Object::Bool(b) => b,
        _ => return Err(format!("Condition must be a boolean: {:?}", cond_obj)),
    };
    if cond {
        eval_obj(&list[2], env)
    } else {
        eval_obj(&list[3], env)
    }
}

fn eval_function_definition(
    list: &Vec<Object>,
    _env: &mut Rc<RefCell<Env>>,
) -> Result<Object, String> {
    let params = match &list[1] {
        Object::List(list) => {
            let mut params = Vec::new();
            for param in list.iter() {
                match param {
                    Object::Symbol(s) => params.push(s.clone()),
                    _ => return Err(format!("Invalid lamdba parameter: {:?}", param)),
                }
            }
            params
        }
        _ => return Err(format!("Invalid lambda parameters: {:?}", list[1])),
    };
    let body = match &list[2] {
        Object::List(list) => list.as_ref().clone(),
        _ => return Err(format!("Invalid lambda body: {:?}", list[2])),
    };
    Ok(Object::Lambda(params, body))
}

fn eval_function_call(
    func_name: &String,
    list: &Rc<Vec<Object>>,
    env: &mut Rc<RefCell<Env>>,
) -> Result<Object, String> {
    let lambda = env.borrow().get(func_name);
    if lambda.is_none() {
        return Err(format!("Undefined function: {}", func_name));
    }
    match lambda.unwrap() {
        Object::Lambda(params, body) => {
            let mut func_env = Rc::new(RefCell::new(Env::extend(Rc::clone(env))));
            for (i, param) in params.iter().enumerate() {
                let arg = eval_obj(&list[i + 1], env)?;
                func_env.borrow_mut().set(param, arg);
            }
            eval_obj(&Object::List(Rc::new(body)), &mut func_env)
        }
        _ => Err(format!("{} is not a function", func_name)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_add() {
        let mut env = Rc::new(RefCell::new(Env::new()));
        let result = eval("(+ 1 2)", &mut env).unwrap();
        assert_eq!(result, Object::Integer(3));
    }

    #[test]
    fn test_circle_area() {
        let mut env = Rc::new(RefCell::new(Env::new()));
        let program = "
        (begin
            (define r 10)
            (define pi 314)
            (* pi (* r r))
        )
        ";

        let result = eval(program, &mut env).unwrap();
        assert_eq!(result, Object::Integer(314 * 10 * 10));
    }

    #[test]
    fn test_srq_function() {
        let mut env = Rc::new(RefCell::new(Env::new()));
        let program = "
        (begin
            (define sqr (lambda (x) (* x x)))
            (sqr 10)
        )
        ";

        let result = eval(program, &mut env).unwrap();
        assert_eq!(result, Object::Integer(100));
    }

    #[test]
    fn test_fibonacci() {
        let mut env = Rc::new(RefCell::new(Env::new()));
        let program = "
        (begin
            (define fib
                (lambda (n)
                    (if (< n 2)
                        n
                        (+ (fib (- n 1)) (fib (- n 2)))
                    )
                )
            )
            (fib 10)
        )
        ";

        let result = eval(program, &mut env).unwrap();
        assert_eq!(result, Object::Integer(55));
    }
}
