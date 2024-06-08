use crate::value::Value;
use std::rc::Rc;

#[derive(Debug, Clone)]
struct FunctionData {
    name: String,
    args_count: usize,
    ptr: Rc<fn(Vec<Value>) -> Option<Value>>,
}
