mod object;
mod types;
mod ast;
mod error;
mod symbol;

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::{RefCell, Cell, Ref};
use std::ops::DerefMut;
use symbol::SymbolId;
use std::any::Any;
use error::Error;
use error::Result;
use types::Value;
use object::Object;


#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub struct ObjectId(usize);

pub struct VirtualMachine {
    object_heap: RefCell<HashMap<ObjectId, Rc<Object>>>,
    next_object_id: Cell<usize>,
    object_assigns_table: RefCell<HashMap<SymbolId, ObjectId>>,
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self {
            object_heap: RefCell::new(HashMap::new()),
            next_object_id: Cell::new(0),
            object_assigns_table: RefCell::new(HashMap::new()),
        }
    }
    pub fn call_method(&self, this: &Value, method: SymbolId, args: &Vec<Value>) -> Result<Value> {
        match this {
            Value::ObjectReference(sid) => {
                let obj = {
                    let obj_heap = self.object_heap.borrow();
                    obj_heap.get(&sid).unwrap().clone()
                };
                let method = obj.get_method(method)?;
                method(&obj, args, self)
            }
            _ => {
                Err(Error::Runtime)
            }
        }
    }

    pub fn assign(&self, target: SymbolId, value: &Value) -> Result<()> {
        match value {
            Value::ObjectReference(oid) => {
                self.object_assigns_table.borrow_mut().insert(target, *oid);
            }
            _ => {
                return Err(Error::Runtime)
            }
        };
        Ok(())
    }

    pub fn allocate(&self, object: Object) -> Result<ObjectId> {
        let id = ObjectId(self.next_object_id.get());
        self.next_object_id.set(id.0 + 1);
        self.object_heap.borrow_mut()
            .insert(id, Rc::new(object));
        Ok(id)
    }

    pub fn get_object(&self, object_id: ObjectId) -> Result<Rc<Object>> {
        self.object_heap.borrow()
            .get(&object_id)
            .ok_or(Error::ObjectNotFound).map(|x| x.clone())
    }
}


#[cfg(test)]
mod tests {
    use crate::{VirtualMachine};
    use crate::ast::{Assign, MethodCall, Decl, ASTNode};
    use crate::symbol::{SymbolId, SymbolTable};
    use crate::object;
    use std::rc::Rc;
    use std::cell::RefCell;
    use crate::object::{ObjectBody, Object};
    use crate::types::Value;

    fn setup() -> (VirtualMachine, SymbolTable) {
        let mut symbol_table = SymbolTable::new();

        let create_symbol = symbol_table.insert_system_symbol("作る");
        let mut root = Object::empty();
        root.add_method(create_symbol, object::root::create);
        let vm = VirtualMachine::new();

        let root_obj_id = vm.allocate(root).unwrap();
        let root_symbol = symbol_table.insert_system_symbol("ルート");
        vm.assign(root_symbol, &Value::ObjectReference(root_obj_id)).unwrap();

        let mut turtle = Object::new(
                ObjectBody::new(
                    &Some(vm.get_object(root_obj_id).unwrap().clone())),
        );
        let turtle_obj_id = vm.allocate(turtle).unwrap();
        let turtle_symbol = symbol_table.insert_system_symbol("タートル");
        vm.assign(turtle_symbol, &Value::ObjectReference(turtle_obj_id)).unwrap();

        (vm, symbol_table)
    }

    #[test]
    fn create() {
        let (vm, st) = setup();
        assert_eq!(vm.object_heap.borrow().len(), 2);
        MethodCall {
            method_symbol: st.get("作る").unwrap(),
            object: Box::new(Decl{target: st.get("ルート").unwrap()}),
            args: vec![],
        }.eval(&vm);

        assert_eq!(vm.object_heap.borrow().len(), 3);
    }

    #[test]
    fn call_parent_method() {
        let (vm, st) = setup();
        assert_eq!(vm.object_heap.borrow().len(), 2);
        MethodCall {
            method_symbol: st.get("作る").unwrap(),
            object: Box::new(Decl{target: st.get("ルート").unwrap()}),
            args: vec![],
        }.eval(&vm);

        assert_eq!(vm.object_heap.borrow().len(), 3);
    }
}
