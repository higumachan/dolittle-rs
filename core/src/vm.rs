use std::rc::Rc;
use crate::symbol::{SymbolId, SymbolTable};
use std::cell::{RefCell, Cell, Ref};
use std::collections::HashMap;
use crate::error::{Error, Result};
use crate::types::Value;
use crate::object::Object;

#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub struct ObjectId(usize);

pub struct VirtualMachine {
    object_heap: RefCell<HashMap<ObjectId, Rc<Object>>>,
    next_object_id: Cell<usize>,
    object_assigns_table: RefCell<HashMap<SymbolId, ObjectId>>,
    symbol_table: RefCell<SymbolTable>,
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self {
            object_heap: RefCell::new(HashMap::new()),
            next_object_id: Cell::new(0),
            object_assigns_table: RefCell::new(HashMap::new()),
            symbol_table: RefCell::new(SymbolTable::new()),
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

    pub fn get_object_id(&self, symbol_id: SymbolId) -> Result<ObjectId> {
        let assigns_table = self.object_assigns_table
            .borrow();
        assigns_table.get(&symbol_id)
            .copied()
            .ok_or(Error::ObjectNotFound)
    }

    pub fn object_heap_borrow(&self) -> Ref<HashMap<ObjectId, Rc<Object>>>{
        self.object_heap.borrow()
    }

    pub fn to_symbol(&self, symbol_str: &str) -> SymbolId {
        self.symbol_table.borrow_mut().insert_user_symbol_if_no_exist(symbol_str)
    }
}
