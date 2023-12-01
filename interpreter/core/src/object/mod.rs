use crate::symbol::SymbolId;
use crate::types::Value;
use crate::error::{Error, Result};
use std::collections::HashMap;
use std::any::{Any};
use crate::vm::{VirtualMachine, ObjectId};
use std::fmt::{Debug, Formatter};
use std::sync::{RwLock, Arc};

type Method = fn(&Value, &Vec<Value>, &VirtualMachine) -> Result<Value>;


#[derive(Debug)]
pub struct Object {
    id: ObjectId,
    body: RwLock<ObjectBody>,
}

impl Object {
    pub fn empty(id: ObjectId) -> Self {
        Self {
            id,
            body: RwLock::new(ObjectBody::new(&None))
        }
    }

    pub fn new(id: ObjectId, body: ObjectBody) -> Self {
        Self {
            id,
            body: RwLock::new(body)
        }
    }

    pub fn is_subclass(&self, obj_id: ObjectId) -> bool {
        if let Some(this_obj) = self.body.read().unwrap().parent.clone() {
            if this_obj.id == obj_id {
                return true;
            }

            this_obj.is_subclass(obj_id)
        } else {
            return false;
        }
    }

    pub fn get_method(&self, symbol: SymbolId) -> Result<Method> {
        self.body.read().unwrap().get_method(symbol)
    }

    pub fn add_method(&self, symbol: SymbolId, method: Method) {
        self.body.write().unwrap().add_method(symbol, method)
    }

    pub fn add_method_str(&self, symbol: &str, method: Method, vm: &VirtualMachine) {
        self.add_method(vm.to_symbol(symbol), method)
    }

    pub fn set_member(&self, symbol: SymbolId, value: Value) {
        self.body.write().unwrap().set_member(symbol, value)
    }

    pub fn get_member(&self, symbol: SymbolId) -> Result<Value> {
        self.body.read().unwrap().get_member(symbol)
    }

    pub fn get_member_str(&self, symbol: &str, vm: &VirtualMachine) -> Result<Value> {
        self.get_member(vm.to_symbol(symbol))
    }

    pub fn set_member_str(&self, symbol: &str, value: Value, vm: &VirtualMachine) {
        self.set_member(vm.to_symbol(symbol), value)
    }

    pub fn set_internal_value(&self, internal_value: Arc<dyn Any + Send + Sync>) {
        self.body.write().unwrap().set_internal_value(internal_value)
    }

    pub fn get_internal_value<T: Clone + Any + Send + Sync>(&self) -> Arc<T> {
        self.body
            .write().unwrap().get_internal_value()
    }
}

pub struct ObjectBody {
    parent: Option<Arc<Object>>,
    members: HashMap<SymbolId, Value>,
    methods: HashMap<SymbolId, Method>,
    internal_value: Option<Arc<dyn Any + Send + Sync>>,
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
    pub fn new(parent: &Option<Arc<Object>>) -> Self {
        ObjectBody{
            parent: parent.clone(),
            members: parent.as_ref().map(
                |p| p.body.read().unwrap().members.clone()
            ).unwrap_or(HashMap::new()),
            methods: HashMap::new(),
            internal_value: None,
        }
    }

    pub fn empty() -> Self {
        ObjectBody::new(&None)
    }

    pub fn get_method(&self, symbol: SymbolId) -> Result<Method> {
        let parent = self.parent.clone();
        Ok(self.methods
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

    pub fn add_method(&mut self, symbol: SymbolId, method: Method) {
        self.methods.insert(symbol, method);
    }

    pub fn add_method_str(&mut self, symbol: &str, method: Method, vm: &VirtualMachine) {
        self.add_method(vm.to_symbol(symbol), method)
    }

    pub fn set_member(&mut self, symbol: SymbolId, value: Value) {
        self.members.insert(symbol, value);
    }

    pub fn get_member(&self, symbol: SymbolId) -> Result<Value> {
        let parent = self.parent.clone();

        Ok(self.members
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

    pub fn set_member_str(&mut self, symbol: &str, value: Value, vm: &VirtualMachine) {
        self.set_member(vm.to_symbol(symbol), value)
    }

    pub fn set_internal_value(&mut self, internal_value: Arc<dyn Any + Send + Sync>) {
        self.internal_value = Some(internal_value);
    }

    pub fn get_internal_value<T: Clone + Any + Send + Sync>(&self) -> Arc<T> {
        self.internal_value.clone()
            .expect("invalid get internal value").downcast::<T>()
            .expect("invalid get internal value").clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::object::{ObjectBody, Object};
    use crate::vm::ObjectId;
    use std::sync::Arc;

    #[test]
    fn is_subclass() {
        let parent = Object::new(ObjectId(0), ObjectBody::empty());
        let child = Object::new(ObjectId(1), ObjectBody::new(&Some(Arc::new(parent))));

        assert!(child.is_subclass(ObjectId(0)));
        assert!(!child.is_subclass(ObjectId(3)));
    }
}

pub mod root {
    use crate::types::Value;
    use crate::vm::VirtualMachine;
    use crate::error::Result;
    use crate::object::{Object, ObjectBody};
    use std::sync::RwLock;

    pub fn create(this: &Value, _args: &Vec<Value>, vm: &VirtualMachine) -> Result<Value> {
        let this_obj = vm.get_object_from_value(this)?;
        let new_object = ObjectBody::new(&Some(this_obj.clone()));
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
            &Value::ObjectReference(vm.get_object_id_in_assigns(
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
    use crate::ast::{ASTNode};
    use std::borrow::Borrow;
    use std::sync::Arc;

    type BlockInternalValue = (Vec<String>, Vec<Arc<ASTNode>>);

    pub fn create(this: &Value, dummy_args: &Vec<String>,
                  body: &Vec<Arc<ASTNode>>, vm: &VirtualMachine) -> Result<Value> {
        let obj_value: Value = super::root::create(this, &vec![], vm)?;
        let obj: Arc<super::Object> = vm.get_object_from_value(&obj_value)?;
        let v = Arc::new(
            (dummy_args.clone(),
             body.clone()
            ));
        obj.set_internal_value(v);
        Ok(obj_value)
    }

    pub fn empty_block(vm: &VirtualMachine) -> Result<Value> {
        let block = vm.get_value_in_scope_from_symbol("ブロック").expect("not defined ブロック");
        let obj_value: Value = super::root::create(&block, &vec![], vm)?;
        let obj: Arc<super::Object> = vm.get_object_from_value(&obj_value)?;
        let v: Arc<(Vec<String>, Vec<Arc<ASTNode>>)> = Arc::new(
            (vec![],
             vec![],
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

    pub fn if_(this: &Value, _args: &Vec<Value>, vm: &VirtualMachine) -> Result<Value> {
        let object_id = super::condition::create_internal(vm)?;
        let object = vm.get_object(object_id).unwrap();
        let flag = exec(this, &vec![], vm)?.as_bool()?;
        object.set_member_str("flag", Value::Bool(flag), vm);

        Ok(Value::ObjectReference(object_id))
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

pub mod condition {
    use crate::vm::{VirtualMachine, ObjectId};
    use crate::types::Value;
    use crate::error::{Result};
    use crate::object::Object;
    use crate::object::root::create;
    use std::sync::Arc;

    pub fn create_super_object(root_object_id: ObjectId, vm: &VirtualMachine) -> Result<ObjectId> {
        let root_value = Value::ObjectReference(root_object_id);
        let super_object_value: Value = super::root::create(&root_value, &vec![], vm)?;
        let super_object: Arc<Object> = super_object_value.as_object(vm)?;

        super_object.set_member_str("flag", Value::Bool(false), vm);
        super_object.add_method_str("実行", exec, vm);
        super_object.add_method_str("そうでないなら", else_, vm);
        let _ = vm.assign(vm.to_symbol("Condition"), &super_object_value);
        super_object_value.as_object_id()
    }

    pub fn create_internal(vm: &VirtualMachine) -> Result<ObjectId> {
        let v = vm.get_value_in_scope_from_symbol("Condition")?;
        create(&v, &vec![], vm)?.as_object_id()
    }

    pub fn exec(this: &Value, args: &Vec<Value>, vm: &VirtualMachine) -> Result<Value> {
        assert_eq!(args.len(), 1);

        let this_obj = vm.get_object_from_value(this)?;
        let b = this_obj.get_member_str("flag", vm)?.as_bool()?;

        if b {
            return super::block::exec(&args[0], &vec![], vm);
        }
        Ok(this.clone())
    }

    pub fn else_(this: &Value, args: &Vec<Value>, vm: &VirtualMachine) -> Result<Value> {
        assert_eq!(args.len(), 0);

        let this_obj = vm.get_object_from_value(this)?;
        let b = this_obj.get_member_str("flag", vm)?.as_bool()?;

        let object_id = create_internal(vm)?;
        let object = vm.get_object(object_id).unwrap();
        object.set_member_str("flag", Value::Bool(!b), vm);

        Ok(Value::ObjectReference(object_id))
    }
}

pub mod button {
    use crate::vm::{ObjectId, VirtualMachine};
    use crate::types::Value;
    use std::sync::Arc;
    use crate::object::Object;
    use crate::error::Result;

    pub fn create_super_object(root_object_id: ObjectId, vm: &VirtualMachine) -> Result<ObjectId> {
        let root_value = Value::ObjectReference(root_object_id);
        let super_object_value: Value = super::root::create(&root_value, &vec![], vm)?;
        let super_object: Arc<Object> = super_object_value.as_object(vm)?;

        super_object.set_member_str("動作", super::block::empty_block(vm)?, vm);
        super_object.add_method_str("クリック", click, vm);

        let _ = vm.assign(vm.to_symbol("ボタン"), &super_object_value);
        super_object_value.as_object_id()
    }

    pub fn click(this: &Value, _args: &Vec<Value>, vm: &VirtualMachine) -> Result<Value> {
        let this_object: Arc<Object> = this.as_object(vm)?;
        let dousa = this_object.get_member_str("動作", vm).expect("not defined 動作");

        super::block::exec(&dousa, &vec![], vm)
    }
}