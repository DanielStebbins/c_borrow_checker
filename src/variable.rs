use std::collections::HashSet;

pub struct Id {
    pub name: String,
    pub scope: usize,
}

impl Clone for Id {
    fn clone(&self) -> Self {
        Id {
            name: self.name.clone(),
            scope: self.scope,
        }
    }
}

pub struct Variable {
    pub id: Id,
    pub is_valid: bool,
    pub is_copy_type: bool,
    pub const_refs: HashSet<Id>,
    pub mut_ref: Option<Id>,
    pub points_to: Option<Id>,
}

impl Variable {
    pub fn new(name: String, scope: usize) -> Self {
        Variable {
            id: Id { name, scope },
            is_valid: true,
            is_copy_type: false,
            const_refs: HashSet::new(),
            mut_ref: None,
            points_to: None,
        }
    }
}

impl Clone for Variable {
    fn clone(&self) -> Self {
        Variable {
            id: self.id.clone(),
            is_valid: self.is_valid,
            is_copy_type: self.is_copy_type,
            const_refs: self.const_refs.clone(),
            mut_ref: self.mut_ref.clone(),
            points_to: self.points_to.clone(),
        }
    }
}
