use crate::vm::ObjectId;

#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    Num(f64),
    Str(String),
    ObjectReference(ObjectId),
    Null,
}


