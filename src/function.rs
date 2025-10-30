use std::{cell::RefCell, rc::Rc};
use crate::{ast::stmt::*, callable::Callable, environment::Environment, interpreter::*};

#[derive(PartialEq)]
pub struct Function {
    decl: FunDecl,
    closure: Rc<RefCell<Environment>>
}

impl Function {
    pub fn new(decl: FunDecl, closure: Rc<RefCell<Environment>>) -> Self {
        Self {
            decl: decl,
            closure: closure
        }
    }
}

impl Callable for Function {
    fn call(&self, interpreter: &mut Interpreter, arguments: Vec<Value>) -> Value {
        let environment = Rc::new(RefCell::new(Environment::new(Some(self.closure.clone()))));
        for i in 0..self.decl.params.len() {
            environment.borrow_mut().define(
                self.decl.params.get(i).unwrap().text.clone(), 
                arguments.get(i).unwrap().to_owned());
        }
        if let Stmt::Block(block) = &*self.decl.body {
            let result = interpreter.execute_block(&block.stmts, environment);
            if let Err(ErrType::Return(value)) = result {
                return value;
            }
        }
        Value::Nil
    }

    fn arity(&self) -> usize {
        self.decl.params.len()
    }
}

impl ToString for Function {
    fn to_string(&self) -> String {
        format!("<fn {}>", self.decl.name.text)
    }
}