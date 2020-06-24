use std::rc::Rc;
use crate::symbol::{SymbolId, SymbolTable};
use std::cell::{RefCell, Cell, Ref};
use std::collections::HashMap;
use crate::error::{Error, Result};
use crate::types::Value;
use crate::object::Object;
use crate::object;
use crate::ast::{ASTNode, Eval};

#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub struct ObjectId(usize);

pub struct VirtualMachine {
    object_heap: RefCell<HashMap<ObjectId, Rc<Object>>>,
    next_object_id: Cell<usize>,
    object_assigns_table: RefCell<HashMap<SymbolId, ObjectId>>,
    symbol_table: RefCell<SymbolTable>,
    stack: RefCell<Vec<HashMap<SymbolId, Value>>>,
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self {
            object_heap: RefCell::new(HashMap::new()),
            next_object_id: Cell::new(0),
            object_assigns_table: RefCell::new(HashMap::new()),
            symbol_table: RefCell::new(SymbolTable::new()),
            stack: RefCell::new(vec![]),
        }
    }

    pub fn eval(&self, ast: &ASTNode) -> Result<Value> {
        ast.eval(self)
    }

    pub fn push_stack(&self, dummy_args: &Vec<String>, real_args: &Vec<Value>) {
        let mut s = HashMap::new();
        for (va, ra) in dummy_args.iter().zip(real_args.iter()) {
            s.insert(self.to_symbol(va), ra.clone());
        }
        self.stack.borrow_mut().push(s);
    }

    pub fn pop_stack(&self) {

    }

    pub fn call_method(&self, this: &Value, method: SymbolId, args: &Vec<Value>) -> Result<Value> {
        match this {
            Value::ObjectReference(oid) => {
                let obj = {
                    let obj_heap = self.object_heap.borrow();
                    obj_heap.get(&oid).unwrap().clone()
                };
                let method = obj.get_method(method)?;
                method(this, args, self)
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

    pub fn get_block_object_value(&self) -> Result<Value> {
        Ok(Value::ObjectReference(self.get_object_id(self.to_symbol("ブロック"))?))
    }

    pub fn get_object_from_value(&self, value :&Value) -> Result<Rc<Object>> {
        match value {
            Value::ObjectReference(obj_id) => {
                self.get_object(*obj_id)
            }
            _ => {
                Err(Error::Runtime)
            }
        }
    }

    pub fn get_object_from_symbol(&self, symbol: &str) -> Result<Rc<Object>> {
        self.get_object(self.get_object_id(self.to_symbol(symbol))?)
    }

    pub fn get_object_heap(&self) -> Ref<HashMap<ObjectId, Rc::<Object>>> {
        self.object_heap.borrow()
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

    pub fn initialize(&mut self) {
        let root_obj_id = {
            let mut root = Object::empty();
            root.add_method(self.to_symbol("作る"), object::root::create);
            let root_obj_id = self.allocate(root).unwrap();
            self.assign(self.to_symbol("ルート"), &Value::ObjectReference(root_obj_id)).unwrap();
            root_obj_id
        };

        let block_obj_id = {
            let block_value = &object::root::create(
                &Value::ObjectReference(root_obj_id), &vec![], self
            ).unwrap().clone();
            let mut block = self.get_object_from_value(
                &block_value
            ).unwrap();

            block.add_method(
                self.to_symbol("実行"),
                object::block::exec
            );

            let block_symbol = self.to_symbol("ブロック");
            self.assign(block_symbol, &block_value).unwrap();

            block_value.as_object_id().unwrap()
        };

        let turtle_obj_id = {
            let turtle_value = &object::root::create(
                &Value::ObjectReference(root_obj_id), &vec![], self
            ).unwrap().clone();
            let mut turtle = self.get_object_from_value(
                &turtle_value
            ).unwrap();
            turtle.add_method(self.to_symbol("歩く"),
                              object::turtle::walk);
            turtle.add_method(self.to_symbol("右回り"),
                              object::turtle::turn_right);
            turtle.add_method(self.to_symbol("左回り"),
                              object::turtle::turn_left);
            turtle.add_method(self.to_symbol("作る"),
                                             object::turtle::create);
            turtle.set_member(self.to_symbol("x"), Value::Num(0.0));
            turtle.set_member(self.to_symbol("y"), Value::Num(0.0));
            turtle.set_member(self.to_symbol("direction"), Value::Num(0.0));
            turtle.set_member(self.to_symbol("visible"),
                              Value::Bool(false));
            let turtle_symbol = self.to_symbol("タートル");
            self.assign(turtle_symbol, &turtle_value).unwrap();
            turtle_value.as_object_id().unwrap()
        };

        let line_obj_id = {
            let line_value = &object::root::create(
                &Value::ObjectReference(root_obj_id), &vec![], self
            ).unwrap().clone();
            let line_symbol = self.to_symbol("線");
            self.assign(line_symbol, &line_value).unwrap();
            line_value.as_object_id().unwrap()
        };
    }
}
