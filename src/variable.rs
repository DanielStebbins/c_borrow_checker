use std::collections::HashSet;

#[derive(Clone)]
pub enum VarType {
    Owner(bool),
    ConstRef(HashSet<Id>),
    MutRef(HashSet<Id>),
}

#[derive(Debug, Clone)]
pub struct Id {
    pub name: String,
    pub scope: usize,
}

// impl Clone for Id {
//     fn clone(&self) -> Self {
//         Id {
//             name: self.name.clone(),
//             scope: self.scope,
//         }
//     }
// }

pub struct Variable {
    pub id: Id,
    pub is_valid: bool,
    pub var_type: VarType,
    pub const_refs: HashSet<Id>,
    pub mut_refs: HashSet<Id>,
}

impl Variable {
    pub fn new(name: String, scope: usize, var_type: VarType) -> Self {
        Variable {
            id: Id { name, scope },
            is_valid: true,
            var_type: var_type,
            const_refs: HashSet::new(),
            mut_refs: HashSet::new(),
        }
    }

    pub fn new_owner(name: String, scope: usize) -> Self {
        Variable {
            id: Id { name, scope },
            is_valid: true,
            var_type: VarType::Owner(true),
            const_refs: HashSet::new(),
            mut_refs: HashSet::new(),
        }
    }

    pub fn new_const_ref(name: String, scope: usize) -> Self {
        let mut variable = Variable::new_owner(name, scope);
        variable.var_type = VarType::ConstRef(HashSet::new());
        variable
    }

    pub fn new_mut_ref(name: String, scope: usize) -> Self {
        let mut variable = Variable::new_owner(name, scope);
        variable.var_type = VarType::MutRef(HashSet::new());
        variable
    }
}

impl Clone for Variable {
    fn clone(&self) -> Self {
        Variable {
            id: self.id.clone(),
            is_valid: self.is_valid,
            var_type: self.var_type.clone(),
            const_refs: self.const_refs.clone(),
            mut_refs: self.mut_refs.clone(),
        }
    }
}
