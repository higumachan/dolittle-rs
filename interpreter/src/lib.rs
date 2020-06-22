use core::vm::VirtualMachine;
use core::ast::ASTNode;

struct Iterpreter {
    vm: VirtualMachine,
}

impl Iterpreter {
    pub fn exec(&self, program: &str) {
        let asts: Vec<ASTNode> = parser::parse_program_code(program);
        for ast in asts {
            vm.eval(ast)
        }
    }
}

#[cfg(test)]
mod tests {
    use core::vm::VirtualMachine;
    use utilities::test_helper::nearly_equal;

    #[test]
    fn test_kameta_create_and_walk() {
        interpreter = Interpreter::new();

        interpreter.exec(r#"かめた＝タートル！作る。
かめた！１００　歩く。
"#);

        let vm: &VirtualMachine = interpreter.get_virtual_machine();
        let kameta = vm.get_object_from_symbol("かめた").unwrap();
        assert!(100.0, kameta.get_member_str("x", vm).unwrap().as_num());
        assert!(0.0, kameta.get_member_str("y", vm).unwrap().as_num());
        assert!(0.0, kameta.get_member_str("direction", vm).unwrap().as_num());
    }

    #[test]
    fn test_kameta_create_and_turnleft90_and_walk() {
        interpreter = Interpreter::new();

        interpreter.exec(r#"かめた＝タートル！作る。
かめた！１００　歩く ９０　左回り。
"#);

        let vm: &VirtualMachine = interpreter.get_virtual_machine();
        let kameta = vm.get_object_from_symbol("かめた").unwrap();
        assert!(0.0, kameta.get_member_str("x", vm).unwrap().as_num());
        assert!(100.0, kameta.get_member_str("y", vm).unwrap().as_num());
        assert!(90.0, kameta.get_member_str("direction", vm).unwrap().as_num());
    }
}
