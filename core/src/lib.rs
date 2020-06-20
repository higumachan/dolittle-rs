mod vm;
mod object;
mod types;
mod ast;
mod error;
mod symbol;

#[cfg(test)]
mod tests {
    use crate::ast::{Assign, MethodCall, Decl, ASTNode};
    use crate::symbol::{SymbolId, SymbolTable};
    use crate::object;
    use std::rc::Rc;
    use std::cell::RefCell;
    use crate::object::{ObjectBody, Object};
    use crate::types::Value;
    use crate::vm::VirtualMachine;

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
        assert_eq!(vm.object_heap_borrow().len(), 2);
        MethodCall {
            method_symbol: st.get("作る").unwrap(),
            object: Box::new(Decl{target: st.get("ルート").unwrap()}),
            args: vec![],
        }.eval(&vm);

        assert_eq!(vm.object_heap_borrow().len(), 3);
    }

    #[test]
    fn call_parent_method() {
        let (vm, st) = setup();
        assert_eq!(vm.object_heap_borrow().len(), 2);
        MethodCall {
            method_symbol: st.get("作る").unwrap(),
            object: Box::new(Decl{target: st.get("ルート").unwrap()}),
            args: vec![],
        }.eval(&vm);

        assert_eq!(vm.object_heap_borrow().len(), 3);
    }
}
