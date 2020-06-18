use std::collections::HashMap;

#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub struct SymbolId(usize);

impl SymbolId {
    fn null() -> Self { Self(0) }
    fn num() -> Self { Self(1) }
    fn str() -> Self { Self(2) }
}

const USER_SYMBOL_START: usize = 10000usize;

pub struct SymbolTable {
    forward: HashMap<String, SymbolId>,
    user_next: usize,
    system_next: usize,
}


impl SymbolTable {
    pub fn new() -> Self {
        let forward = HashMap::new();

        Self {
            forward,
            user_next: USER_SYMBOL_START,
            system_next: 0usize,
        }
    }

    pub fn get(&self, name: &str) -> Option<SymbolId> {
        self.forward.get(name).copied()
    }

    pub fn insert_system_symbol(&mut self, name: &str) -> SymbolId {
        let sid = SymbolId(self.system_next);
        self.forward.insert(name.to_string(), sid);
        self.system_next += 1;
        sid
    }

    pub fn insert_user_symbol(&mut self, name: &str) {
        self.forward.insert(name.to_string(), SymbolId(self.user_next));
        self.user_next += 1;
    }
}
