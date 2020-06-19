use crate::ObjectId;

#[derive(Clone, Debug)]
pub enum Value {
    Num(f64),
    Str(String),
    ObjectReference(ObjectId),
    Null,
}


