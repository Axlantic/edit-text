//! Enables stepping through a span operation.

use std::collections::HashMap;
use doc::*;
use std::borrow::ToOwned;
use std::cmp;

#[derive(Clone, Debug)]
pub struct DelStepper {
    pub head: Option<DelElement>,
    pub rest: Vec<DelElement>,
    pub stack: Vec<Vec<DelElement>>,
}

impl DelStepper {
    pub fn new(span: &DelSpan) -> DelStepper {
        let mut ret = DelStepper {
            head: None,
            rest: span.to_vec(),
            stack: vec![],
        };
        ret.next();
        ret
    }

    pub fn next(&mut self) -> Option<DelElement> {
        let res = self.head.clone();
        self.head = if self.rest.len() > 0 {
            Some(self.rest.remove(0))
        } else {
            None
        };
        res
    }

    pub fn get_head(&self) -> DelElement {
        self.head.clone().unwrap()
    }

    pub fn is_done(&self) -> bool {
        self.head.is_none() && self.stack.len() == 0
    }

    pub fn enter(&mut self) {
        let head = self.head.clone();
        self.stack.push(self.rest.clone());
        match head {
            Some(DelGroup(ref span)) | Some(DelWithGroup(ref span)) => {
                self.head = None;
                self.rest = span.to_vec();
                self.next();
            }
            _ => panic!("Entered wrong thing"),
        }
    }

    pub fn exit(&mut self) {
        let last = self.stack.pop().unwrap();
        self.rest = last;
        self.next();
    }

    pub fn into_span(self) -> DelSpan {
        let DelStepper { head, rest, .. } = self;
        if let Some(head) = head {
            let mut out = rest.to_vec();
            out.insert(0, head);
            out
        } else {
            vec![]
        }
    }
}


#[derive(Clone, Debug)]
pub struct AddStepper {
    pub head: Option<AddElement>,
    pub rest: Vec<AddElement>,
    pub stack: Vec<Vec<AddElement>>,
}

impl AddStepper {
    pub fn new(span: &AddSpan) -> AddStepper {
        let mut ret = AddStepper {
            head: None,
            rest: span.to_vec(),
            stack: vec![],
        };
        ret.next();
        ret
    }

    pub fn next(&mut self) -> Option<AddElement> {
        let res = self.head.clone();
        self.head = if self.rest.len() > 0 {
            Some(self.rest.remove(0))
        } else {
            None
        };
        res
    }

    pub fn get_head(&self) -> AddElement {
        self.head.clone().unwrap()
    }

    pub fn is_done(&self) -> bool {
        self.head.is_none() && self.stack.len() == 0
    }

    pub fn into_span(self) -> AddSpan {
        let AddStepper { head, rest, .. } = self;
        if let Some(head) = head {
            let mut out = rest.to_vec();
            out.insert(0, head);
            out
        } else {
            vec![]
        }
    }

    pub fn enter(&mut self) {
        let head = self.head.clone();
        self.stack.push(self.rest.clone());
        match head {
            Some(AddGroup(_, ref span)) | Some(AddWithGroup(ref span)) => {
                self.head = None;
                self.rest = span.to_vec();
                self.next();
            }
            _ => panic!("Entered wrong thing"),
        }
    }

    pub fn exit(&mut self) {
        let last = self.stack.pop().unwrap();
        self.rest = last;
        self.next();
    }
}
