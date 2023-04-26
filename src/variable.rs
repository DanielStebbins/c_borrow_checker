use std::collections::HashSet;

#[derive(Clone)]
pub enum VarType {
    Owner(bool),
    ConstRef(HashSet<Id>),
    MutRef(HashSet<Id>),
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct Id {
    pub name: String,
    pub scope: usize,
}

pub struct Variable {
    pub id: Id,
    pub var_type: VarType,
    pub const_refs: HashSet<Id>,
    pub mut_refs: HashSet<Id>,
}

impl Variable {
    pub fn new(name: String, scope: usize, var_type: VarType) -> Self {
        Variable {
            id: Id { name, scope },
            var_type: var_type,
            const_refs: HashSet::new(),
            mut_refs: HashSet::new(),
        }
    }

    pub fn new_owner(name: String, scope: usize) -> Self {
        Variable {
            id: Id { name, scope },
            var_type: VarType::Owner(true),
            const_refs: HashSet::new(),
            mut_refs: HashSet::new(),
        }
    }
}

impl Clone for Variable {
    fn clone(&self) -> Self {
        Variable {
            id: self.id.clone(),
            var_type: self.var_type.clone(),
            const_refs: self.const_refs.clone(),
            mut_refs: self.mut_refs.clone(),
        }
    }
}
