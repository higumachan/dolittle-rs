use crate::symbol::{SymbolId, SymbolTable};
use std::collections::HashMap;
use crate::error::{Error, Result};
use crate::types::Value;
use crate::object::{Object, ObjectBody};
use crate::object;
use crate::ast::{ASTNode, Eval};
use std::sync::{RwLock, Arc, Mutex, RwLockReadGuard};

#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub struct ObjectId(usize);

pub struct VirtualMachine {
    object_heap: RwLock<HashMap<ObjectId, Arc<Object>>>,
    next_object_id: Mutex<usize>,
    value_assigns_table: RwLock<HashMap<SymbolId, Value>>,
    symbol_table: RwLock<SymbolTable>,
    stack: RwLock<Vec<HashMap<SymbolId, Value>>>,
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self {
            object_heap: RwLock::new(HashMap::new()),
            next_object_id: Mutex::new(0),
            value_assigns_table: RwLock::new(HashMap::new()),
            symbol_table: RwLock::new(SymbolTable::new()),
            stack: RwLock::new(vec![]),
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
        self.stack.write().unwrap().push(s);
    }

    pub fn pop_stack(&self) {
        self.stack.write().unwrap().pop();
    }

    pub fn call_method(&self, this: &Value, method: SymbolId, args: &Vec<Value>) -> Result<Value> {
        match this {
            Value::ObjectReference(oid) => {
                let obj = {
                    let obj_heap = self.object_heap.read().unwrap();
                    obj_heap.get(&oid).unwrap().clone()
                };
                let method_obj = obj.get_member(method);
                match method_obj {
                    Ok(x) => {
                        object::block::exec(&x,
                                            args, self)
                    }
                    Err(Error::MemberNotFound) => {
                        let method = obj.get_method(method)?;
                        method(this, args, self)
                    }
                    Err(e) => {
                        Err(e)
                    }
                }
            }
            _ => {
                Err(Error::Runtime)
            }
        }
    }

    pub fn assign(&self, target: SymbolId, value: &Value) -> Result<()> {
        self.value_assigns_table.write().unwrap().insert(target, value.clone());
        Ok(())
    }

    pub fn allocate(&self, object_body: ObjectBody) -> Result<ObjectId> {
        let id = ObjectId(*self.next_object_id.lock().unwrap());
        *self.next_object_id.lock().unwrap() = id.0 + 1;

        let object = Object::new(id, object_body);
        self.object_heap.write().unwrap()
            .insert(id, Arc::new(object));
        Ok(id)
    }

    pub fn get_object(&self, object_id: ObjectId) -> Result<Arc<Object>> {
        self.object_heap.read().unwrap()
            .get(&object_id)
            .ok_or(Error::ObjectNotFound).map(|x| x.clone())
    }

    pub fn get_block_object_value(&self) -> Result<Value> {
        Ok(Value::ObjectReference(self.get_object_id_in_assigns(self.to_symbol("ブロック"))?))
    }

    pub fn get_object_from_value(&self, value :&Value) -> Result<Arc<Object>> {
        match value {
            Value::ObjectReference(obj_id) => {
                self.get_object(*obj_id)
            }
            _ => {
                Err(Error::Runtime)
            }
        }
    }

    pub fn get_object_in_assigns_from_symbol(&self, symbol: &str) -> Result<Arc<Object>> {
        self.get_object(
            self.get_object_id_in_assigns(
                self.to_symbol(symbol))?)
    }

    pub fn get_object_heap(&self) -> RwLockReadGuard<HashMap<ObjectId, Arc::<Object>>> {
        self.object_heap.read().unwrap()
    }

    pub fn get_object_id_in_assigns(&self, symbol_id: SymbolId) -> Result<ObjectId> {
        self.get_value_in_assigns(symbol_id)?.as_object_id()
    }

    fn get_value_in_assigns(&self, symbol_id: SymbolId) -> Result<Value> {
        let assigns_table = self.value_assigns_table
            .read().unwrap();
        assigns_table.get(&symbol_id)
            .cloned()
            .ok_or(Error::ObjectNotFound)
    }

    pub fn get_value_in_scope(&self, symbol_id: SymbolId) -> Result<Value> {
        self.stack.read().unwrap().iter().rev().find_map(|x| x.get(&symbol_id).cloned())
            .ok_or(Error::ObjectNotFound)
            .or_else(|_| {
                self.get_value_in_assigns(symbol_id)
            })
    }

    pub fn get_value_in_scope_from_symbol(&self, symbol: &str) -> Result<Value> {
        let sym_id = self.to_symbol(symbol);
        self.get_value_in_scope(sym_id)
    }

    pub fn object_heap_borrow(&self) -> RwLockReadGuard<HashMap<ObjectId, Arc<Object>>>{
        self.object_heap.read().unwrap()
    }

    pub fn to_symbol(&self, symbol_str: &str) -> SymbolId {
        self.symbol_table.write().unwrap().insert_user_symbol_if_no_exist(symbol_str)
    }

    pub fn initialize(&mut self) {
        let root_obj_id = {
            let mut root = ObjectBody::empty();
            root.add_method(self.to_symbol("作る"), object::root::create);
            let root_obj_id = self.allocate(root).unwrap();
            self.assign(self.to_symbol("ルート"), &Value::ObjectReference(root_obj_id)).unwrap();
            root_obj_id
        };

        let _block_obj_id = {
            let block_value = &object::root::create(
                &Value::ObjectReference(root_obj_id), &vec![], self
            ).unwrap().clone();
            let block = self.get_object_from_value(
                &block_value
            ).unwrap();

            block.add_method(
                self.to_symbol("実行"),
                object::block::exec
            );

            block.add_method(
                self.to_symbol("繰り返す"),
                object::block::repeat,
            );

            block.add_method(
                self.to_symbol("ならば"),
                object::block::if_,
            );

            let block_symbol = self.to_symbol("ブロック");
            self.assign(block_symbol, &block_value).unwrap();

            block_value.as_object_id().unwrap()
        };

        let _turtle_obj_id = {
            let turtle_value = &object::root::create(
                &Value::ObjectReference(root_obj_id), &vec![], self
            ).unwrap().clone();
            let turtle = self.get_object_from_value(
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

        let _line_obj_id = {
            let line_value = &object::root::create(
                &Value::ObjectReference(root_obj_id), &vec![], self
            ).unwrap().clone();
            let line_symbol = self.to_symbol("線");
            self.assign(line_symbol, &line_value).unwrap();
            line_value.as_object_id().unwrap()
        };

        let _condition_obj_id = object::condition::create_super_object(
            root_obj_id, self
        ).unwrap();

        let _button_obj_id =
            object::button::create_super_object(root_obj_id, self).unwrap();
    }
}
