use std::rc::Rc;
use crate::interpreter::Value;

#[derive(PartialEq)]
pub struct Array {
    pub elements: Vec<Rc<Value>>
}

impl Array {
    pub fn new(elements: Vec<Rc<Value>>) -> Self {
        Self {
            elements: elements
        }
    }

    pub fn get(&self, mut i: isize) -> Option<Rc<Value>> {
        if i < 0 {
            i = self.elements.len() as isize + i;
        }
        if (i as usize) < self.elements.len() {
            Some(self.elements[i as usize].clone())
        } else {
            None
        }
    }

    pub fn set(&mut self, i: isize, value: Rc<Value>) -> bool {
        let i = if i < 0 {
            self.elements.len() as isize + i
        } else {
            i
        } as usize;
        if i < self.elements.len() {
            self.elements[i] = value;
            true
        } else {
            false
        }
    }

    pub fn push(&mut self, value: Rc<Value>) {
        self.elements.push(value);
    }

    pub fn pop(&mut self) -> Option<Rc<Value>> {
        self.elements.pop()
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }
}

impl ToString for Array {
    fn to_string(&self) -> String {
        let mut result = "[".to_string();
        for i in 0..self.elements.len() {
            if i > 0 {
                result.push_str(", ");
            }
            let element = &self.elements[i];
            result.push_str(&format!("{element}"));
        }
        result.push_str("]");
        result
    }
}

