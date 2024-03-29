use core::vm::{VirtualMachine, ObjectId};
use core::ast::ASTNode;
use core::object::Object;
use std::collections::HashMap;
use std::rc::Rc;
use core::symbol::SymbolId;
use std::sync::Arc;

pub struct Interpreter {
    vm: VirtualMachine,
}

impl Interpreter {
    pub fn exec(&mut self, program: &str) {
        let (_, asts)= parser::parse_program_code(program).unwrap();
        for ast in asts {
            self.vm.eval(&ast).unwrap();
        }
    }

    pub fn get_objects(&self) -> Vec<Arc<Object>> {
        self.vm.get_object_heap().iter()
            .map(|x| x.1.clone()).collect()
    }

    pub fn get_symbol(&self, s: &str) -> SymbolId {
        self.vm.to_symbol(s)
    }

    pub fn get_object_id(&self, symbol: &str) -> ObjectId {
        self.vm.get_value_in_scope_from_symbol(symbol)
            .expect("object not found")
            .as_object_id()
            .expect("object not found")
    }

    pub fn new() -> Self {
        let mut vm = VirtualMachine::new();
        vm.initialize();

        Self {
            vm
        }
    }
}

#[cfg(test)]
mod tests {
    use core::vm::VirtualMachine;
    use crate::Interpreter;
    use utilities::test_helper::{nearly_equal, nearly_equal_with_eps};

    const eps: f64 = 1e-5;

    #[test]
    fn test_kameta_create_and_walk() {
        let mut interpreter = Interpreter::new();

        interpreter.exec(r#"かめた＝タートル！作る。
かめた！１００　歩く。
"#);

        let vm = interpreter.vm;
        let kameta = vm.get_object_in_assigns_from_symbol("かめた").unwrap();
        assert!(nearly_equal(
            100.0, kameta.get_member_str("x", &vm).unwrap().as_num().unwrap()));
        assert!(nearly_equal(0.0, kameta.get_member_str("y", &vm).unwrap().as_num().unwrap()));
        assert!(nearly_equal(0.0, kameta.get_member_str("direction", &vm).unwrap().as_num().unwrap()));
        assert!(kameta.get_member_str("visible", &vm).unwrap().as_bool().unwrap());
    }

    #[test]
    fn test_kameta_create_and_turnleft90_and_walk() {
        let mut interpreter = Interpreter::new();

        interpreter.exec(r#"かめた＝タートル！作る。
かめた！ ９０　左回り １００　歩く。
"#);

        let vm = interpreter.vm;
        let kameta = vm.get_object_in_assigns_from_symbol("かめた").unwrap();

        assert!(nearly_equal_with_eps(
            0.0, kameta.get_member_str("x", &vm).unwrap().as_num().unwrap(), eps));
        assert!(nearly_equal_with_eps(
            100.0, kameta.get_member_str("y", &vm).unwrap().as_num().unwrap(), eps));
        assert!(nearly_equal_with_eps(
            90.0, kameta.get_member_str("direction", &vm).unwrap().as_num().unwrap(), eps));
        assert!(kameta.get_member_str("visible", &vm).unwrap().as_bool().unwrap());
        assert_eq!(vm.get_object_heap().len(), 9);
    }

    #[test]
    fn test_kameta_square() {
        let mut interpreter = Interpreter::new();

        interpreter.exec(r#"かめた＝タートル！作る。
かめた：四角＝「｜長さ｜ かめた！（長さ） 歩く。 かめた！９０ 右回り。」。
かめた！１００　四角。"#);


        let vm = interpreter.vm;
        let kameta = vm.get_object_in_assigns_from_symbol("かめた").unwrap();
        assert!(nearly_equal_with_eps(
            100.0, kameta.get_member_str("x", &vm).unwrap().as_num().unwrap(), eps));
        assert!(nearly_equal_with_eps(
            0.0, kameta.get_member_str("y", &vm).unwrap().as_num().unwrap(), eps));
    }

    #[test]
    fn test_assign_static_value() {
        let mut interpreter = Interpreter::new();

        interpreter.exec("てすと＝１。");

        assert_eq!(interpreter.vm.get_value_in_scope_from_symbol("てすと").unwrap().as_num().unwrap(), 1.0)
    }

    #[test]
    fn test_repeat() {
        let mut interpreter = Interpreter::new();

        interpreter.exec("かめた＝タートル！　作る。");
        interpreter.exec("「かめた！１００　歩く。」！４　繰り返す。");

        let vm = interpreter.vm;
        let kameta = vm.get_object_in_assigns_from_symbol("かめた").unwrap();
        assert!(nearly_equal_with_eps(
            400.0, kameta.get_member_str("x", &vm).unwrap().as_num().unwrap(), eps));
        assert!(nearly_equal_with_eps(
            0.0, kameta.get_member_str("y", &vm).unwrap().as_num().unwrap(), eps));

    }

    #[test]
    fn test_if() {
        let mut interpreter = Interpreter::new();

        interpreter.exec("てすと＝１。");
        assert_eq!(interpreter.vm.get_value_in_scope_from_symbol("てすと").unwrap().as_num().unwrap(), 1.0);
        interpreter.exec("「てすと＝＝１。」！ならば　「てすと２＝２。」　実行。");
        assert_eq!(interpreter.vm.get_value_in_scope_from_symbol("てすと２").unwrap().as_num().unwrap(), 2.0);
        interpreter.exec("「てすと＝＝０。」！ならば　「てすと２＝２。」　実行　そうでないなら　「てすと３＝３。」　実行。");
        assert_eq!(interpreter.vm.get_value_in_scope_from_symbol("てすと３").unwrap().as_num().unwrap(), 3.0);
    }
}
