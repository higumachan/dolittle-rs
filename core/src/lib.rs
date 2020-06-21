mod vm;
mod object;
pub mod types;
pub mod ast;
mod error;
pub mod symbol;

#[cfg(test)]
mod tests {
    use crate::symbol::{SymbolId, SymbolTable};
    use crate::object;
    use std::rc::Rc;
    use std::cell::RefCell;
    use crate::object::{ObjectBody, Object};
    use crate::types::Value;
    use crate::vm::VirtualMachine;
    use crate::ast::{ASTNode, Eval};

    fn setup() -> VirtualMachine {
        let vm = VirtualMachine::new();
        let create_symbol = vm.to_symbol("作る");
        let mut root = Object::empty();
        root.add_method(create_symbol, object::root::create);

        let root_obj_id = vm.allocate(root).unwrap();
        let root_symbol = vm.to_symbol("ルート");
        vm.assign(root_symbol, &Value::ObjectReference(root_obj_id)).unwrap();

        let mut turtle = Object::new(
                ObjectBody::new(
                    &Some(vm.get_object(root_obj_id).unwrap().clone())),
        );
        let turtle_obj_id = vm.allocate(turtle).unwrap();
        let turtle_symbol = vm.to_symbol("タートル");
        vm.assign(turtle_symbol, &Value::ObjectReference(turtle_obj_id)).unwrap();

        vm
    }

    #[test]
    fn create() {
        let vm = setup();
        assert_eq!(vm.object_heap_borrow().len(), 2);
        ASTNode::new_method_call(
            "作る".to_string(),
            ASTNode::new_decl("ルート".to_string()),
            vec![],
        ).eval(&vm);

        assert_eq!(vm.object_heap_borrow().len(), 3);
    }

    #[test]
    fn call_parent_method() {
        let vm = setup();
        assert_eq!(vm.object_heap_borrow().len(), 2);
        ASTNode::new_method_call(
            "作る".to_string(),
            ASTNode::new_decl("タートル".to_string()),
            vec![],
        ).eval(&vm);

        assert_eq!(vm.object_heap_borrow().len(), 3);
    }
}
