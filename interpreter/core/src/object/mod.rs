use std::cell::RefCell;
use crate::symbol::SymbolId;
use std::rc::Rc;
use crate::types::Value;
use crate::error::{Error, Result};
use std::collections::HashMap;
use std::any::Any;
use crate::vm::VirtualMachine;

type Method = fn(&Rc<Object>, &Vec<Value>, &VirtualMachine) -> Result<Value>;

pub struct Object {
    body: RefCell<ObjectBody>,
}

impl Object {
    pub fn empty() -> Self {
        Self {
            body: RefCell::new(ObjectBody::new(&None))
        }
    }

    pub fn new(body: ObjectBody) -> Self {
        Self {
            body: RefCell::new(body)
        }
    }

    pub fn get_method(&self, symbol: SymbolId) -> Result<Method> {
        let parent = self.body.borrow().parent.clone();
        Ok(self.body.borrow().methods
            .get(&symbol)
            .map(|x| x.clone())
            .or_else(|| {
                if let Some(parent) = parent {
                    parent.get_method(symbol).ok()
                } else {
                    None
                }
            })
            .ok_or(Error::MethodNotFound)?)
    }

    pub fn add_method(&self, symbol: SymbolId, method: Method) {
        self.body.borrow_mut().methods.insert(symbol, method);
    }

    pub fn add_member(&self, symbol: SymbolId, value: Value) {
        self.body.borrow_mut().members.insert(symbol, value);
    }
}

pub struct ObjectBody {
    parent: Option<Rc<Object>>,
    members: HashMap<SymbolId, Value>,
    methods: HashMap<SymbolId, Method>,
    internal_values: Option<Box<dyn Any>>,
}

impl ObjectBody {
    pub fn new(parent: &Option<Rc<Object>>) -> Self {
        ObjectBody{
            parent: parent.clone(),
            members: HashMap::new(),
            methods: HashMap::new(),
            internal_values: None,
        }
    }
}


pub mod root {
    use std::cell::RefCell;
    use std::rc::Rc;
    use crate::types::Value;
    use crate::vm::VirtualMachine;
    use crate::error::Result;
    use crate::object::{Object, ObjectBody};

    pub fn create(this: &Value, _args: &Vec<Value>, vm: &VirtualMachine) -> Result<Value> {
        let this_obj = vm.get_object_from_value(this)?;
        let new_object = Object {
            body: RefCell::new(ObjectBody::new(&Some(this_obj.clone())))
        };
        Ok(Value::ObjectReference(vm.allocate(new_object)?))
    }
}

pub mod turtle {
    use crate::types::Value;
    use crate::vm::VirtualMachine;
    use crate::error::Result;
    use utilities::geometry::dir_vector;

    pub fn walk(this: &Value, _args: &Vec<Value>, vm: &VirtualMachine) -> Result<Value> {
        let this_obj = vm.get_object_from_value(this)?;
        let dv = dir_vector(this_obj.get_value("r")?.as_num()?);
        this.obj.set_value(this_obj.get_value("x")?.as_num()? + dv.x);
        this.obj.set_value(this_obj.get_value("y")?.as_num()? + dv.y);

        Ok(this.clone())
    }
}