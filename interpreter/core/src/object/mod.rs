use std::cell::RefCell;
use crate::symbol::SymbolId;
use std::rc::Rc;
use crate::types::Value;
use crate::error::{Error, Result};
use std::collections::HashMap;
use std::any::{Any, TypeId};
use crate::vm::{VirtualMachine, ObjectId};
use std::fmt::{Debug, Formatter};

type Method = fn(&Value, &Vec<Value>, &VirtualMachine) -> Result<Value>;


#[derive(Debug)]
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

    pub fn set_member(&self, symbol: SymbolId, value: Value) {
        self.body.borrow_mut().members.insert(symbol, value);
    }

    pub fn get_member(&self, symbol: SymbolId) -> Result<Value> {
        let parent = self.body.borrow().parent.clone();

        Ok(self.body.borrow().members
            .get(&symbol)
            .map(|x| x.clone())
            .or_else(|| {
                if let Some(parent) = parent {
                    parent.get_member(symbol).ok()
                } else {
                    None
                }
            })
            .ok_or(Error::MemberNotFound)?)
    }

    pub fn get_member_str(&self, symbol: &str, vm: &VirtualMachine) -> Result<Value> {
        self.get_member(vm.to_symbol(symbol))
    }

    pub fn set_member_str(&self, symbol: &str, value: Value, vm: &VirtualMachine) {
        self.set_member(vm.to_symbol(symbol), value)
    }

    pub fn set_internal_value(&self, internal_value: Rc<dyn Any>) {
        self.body.borrow_mut().internal_value = Some(internal_value);
    }

    pub fn get_internal_value<T: Clone + Any>(&self) -> Rc<T> {
        self.body
            .borrow_mut().internal_value.clone()
            .expect("invalid get internal value").downcast::<T>()
            .expect("invalid get internal value").clone()
    }
}

pub struct ObjectBody {
    parent: Option<Rc<Object>>,
    members: HashMap<SymbolId, Value>,
    methods: HashMap<SymbolId, Method>,
    internal_value: Option<Rc<dyn Any>>,
}

impl Debug for ObjectBody {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ObjectBody")
            .field("parent", &format!("{:?}", self.parent))
            .field("members", &self.members)
            .finish()
    }
}

impl ObjectBody {
    pub fn new(parent: &Option<Rc<Object>>) -> Self {
        ObjectBody{
            parent: parent.clone(),
            members: parent.as_ref().map(
                |p| p.body.borrow().members.clone()
            ).unwrap_or(HashMap::new()),
            methods: HashMap::new(),
            internal_value: None,
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
    use crate::error::{Error, Result};
    use utilities::geometry::dir_vector;

    const x: &str = "x";
    const y: &str = "y";
    const direction: &str = "direction";
    const visible: &str = "visible";

    pub fn create(this: &Value, _args: &Vec<Value>, vm: &VirtualMachine) -> Result<Value> {
        let obj_value: Value = super::root::create(this, _args, vm)?;
        let obj = vm.get_object_from_value(&obj_value)?;
        obj.set_member_str(visible, Value::Bool(true), vm);
        Ok(obj_value)
    }

    pub fn walk(this: &Value, args: &Vec<Value>, vm: &VirtualMachine) -> Result<Value> {
        let amount = args.get(0).ok_or(Error::ArgumentError)?.as_num()?;
        let this_obj = vm.get_object_from_value(this)?;
        let dv = dir_vector(this_obj.get_member_str(direction, vm)?.as_num()?);
        let (x1, y1) =
            (this_obj.get_member_str(x, vm)?.as_num()?, this_obj.get_member_str(y, vm)?.as_num()?);
        let (x2, y2) = (x1 + amount * dv.x, y1 + amount * dv.y);
        this_obj.set_member_str(x, Value::Num(x2), vm);
        this_obj.set_member_str(y, Value::Num(y2), vm);

        let line = vm.call_method(
            &Value::ObjectReference(vm.get_object_id(
                vm.to_symbol("線"))?),
            vm.to_symbol("作る"), &vec![])?;
        let line_obj = vm.get_object_from_value(&line)?;
        line_obj.set_member_str("x1", Value::Num(x1), vm);
        line_obj.set_member_str("y1", Value::Num(y1), vm);
        line_obj.set_member_str("x2", Value::Num(x2), vm);
        line_obj.set_member_str("y2", Value::Num(y2), vm);

        Ok(this.clone())
    }

    pub fn turn_left(this: &Value, args: &Vec<Value>, vm: &VirtualMachine) -> Result<Value> {
        let angle_deg = args.get(0).ok_or(Error::ArgumentError)?.as_num()?;
        let this_obj = vm.get_object_from_value(this)?;
        this_obj.set_member_str(direction,
                                Value::Num(this_obj.get_member_str(direction, vm)?.as_num()? + angle_deg), vm);

        Ok(this.clone())
    }

    pub fn turn_right(this: &Value, args: &Vec<Value>, vm: &VirtualMachine) -> Result<Value> {
        let angle_deg = -(args.get(0).ok_or(Error::ArgumentError)?.as_num()?);
        let this_obj = vm.get_object_from_value(this)?;
        this_obj.set_member_str(direction,
                                Value::Num(this_obj.get_member_str(direction, vm)?.as_num()? + angle_deg), vm);

        Ok(this.clone())
    }
}

pub mod block {
    use crate::types::Value;
    use crate::vm::VirtualMachine;
    use crate::error::{Error, Result};
    use utilities::geometry::dir_vector;
    use crate::ast::{ASTNode, BlockDefineImpl};
    use std::rc::Rc;
    use std::borrow::Borrow;
    use std::any::{TypeId, Any};

    type BlockInternalValue = (Vec<String>, Vec<Rc<ASTNode>>);

    pub fn create(this: &Value, dummy_args: &Vec<String>,
                  body: &Vec<Rc<ASTNode>>, vm: &VirtualMachine) -> Result<Value> {
        let obj_value: Value = super::root::create(this, &vec![], vm)?;
        let obj: Rc<super::Object> = vm.get_object_from_value(&obj_value)?;
        let v = Rc::new(
            (dummy_args.clone(),
             body.clone()
            ));
        obj.set_internal_value(v);
        Ok(obj_value)
    }

    pub fn repeat(this: &Value, args: &Vec<Value>, vm: &VirtualMachine) -> Result<Value> {
        let n: f64 = args.first().ok_or(Error::Runtime)?.as_num()?;
        if n.is_sign_negative() {
            return Err(Error::Runtime);
        }
        let n = n.floor() as u64;
        if n < 0 {
            return Err(Error::Runtime);
        }

        for _ in 0..n-1 {
            exec(this, &vec![], vm)?;
        }
        exec(this, &vec![], vm)
    }

    pub fn exec(this: &Value, args: &Vec<Value>, vm: &VirtualMachine) -> Result<Value> {
        let this_obj = vm.get_object_from_value(this)?;
        let t = this_obj.get_internal_value::<BlockInternalValue>();
        let (dummy_args, body) = t.borrow();
        vm.push_stack(dummy_args, args);
        let mut result = Value::Null;
        for b in body.iter() {
            result = vm.eval(b)?;
        }
        vm.pop_stack();
        return Ok(result)
    }
}