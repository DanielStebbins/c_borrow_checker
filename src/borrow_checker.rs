use crate::variable::*;
use lang_c::ast::*;
use lang_c::loc::*;
use lang_c::span::*;
use lang_c::visit::Visit;
use lang_c::*;
use std::collections::HashMap;
use std::collections::HashSet;

pub enum PrintType {
    Ownership,
    Reference,
    ErrorOnly,
}

pub struct BorrowChecker<'a> {
    // For the user to specify what functions the checks should run on.
    pub functions_to_check: Vec<String>,

    pub src: &'a str,
    pub scopes: Vec<HashMap<String, Variable>>,

    // So the checker knows the types of struct members (need to know if they are copy types or not)
    // and function parameters (need to know if they are marked const in the function header).
    pub structs: HashMap<String, HashMap<String, VarType>>,
    pub functions: HashMap<String, Vec<VarType>>,

    // Struct member identifier compilation.
    pub mute_member_expression: bool,
    pub member_count: u32,
    pub member_identifier_pieces: Vec<String>,
    pub member_identifier: String,

    // Function body scope creation is handled in the function definition block to include the parameters.
    // This stops visit_statement from creating another new scope.
    pub function_body: bool,

    // Tracks the previous struct name seen, for when pointers to structs are declared as function parameters.
    pub previous_struct_name: String,

    pub dereference_name: String,

    // Controls what kind of output is shown.
    pub set_prints: PrintType,
    pub event_prints: PrintType,
}

impl<'a> BorrowChecker<'a> {
    pub fn new(
        to_check: Vec<String>,
        source: &'a str,
        set_prints: PrintType,
        event_prints: PrintType,
    ) -> Self {
        BorrowChecker {
            functions_to_check: to_check,

            src: source,
            scopes: vec![HashMap::new()],

            structs: HashMap::new(),
            functions: HashMap::new(),

            mute_member_expression: false,
            member_count: 0,
            member_identifier_pieces: Vec::new(),
            member_identifier: "".to_string(),

            function_body: false,

            previous_struct_name: "".to_string(),

            dereference_name: "".to_string(),

            set_prints: set_prints,
            event_prints: event_prints,
        }
    }
}

// Functions that mutate and print information about the ownership of variables.
impl<'a> BorrowChecker<'a> {
    // Finds the most local (highest count) scope where the given name exists.
    pub fn get_scope_number(&self, mut name: &str) -> usize {
        let mut count: usize = self.scopes.len() - 1;
        if name.contains(".") {
            name = &name[..name.find(".").unwrap()];
        }
        for scope in self.scopes.iter().rev() {
            if scope.contains_key(name) {
                return count;
            }
            if count != 0 {
                count -= 1;
            }
        }
        return count;
    }

    pub fn get_id(&self, name: &str) -> Id {
        return Id {
            name: name.to_string(),
            scope: self.get_scope_number(name),
        };
    }

    pub fn id_to_var(&self, id: &Id) -> &Variable {
        return self.scopes[id.scope].get(&id.name).unwrap();
    }

    pub fn id_to_mut_var(&mut self, id: &Id) -> &mut Variable {
        return self.scopes[id.scope].get_mut(&id.name).unwrap();
    }

    // Assumes the variable is an owner type.
    pub fn name_to_var(&mut self, name: &str) -> &Variable {
        let count = self.get_scope_number(name);
        if !self.scopes[count].contains_key(name) {
            let var_type = self.get_member_var_type(name);
            println!("Created new variable '{name}' of type {:?}", var_type);
            self.scopes[count].insert(
                name.to_string(),
                Variable::new(name.to_string(), 0, var_type.clone()),
            );
            self.declare_unknown_global(name, var_type, false)
        }
        return self.scopes[count].get(name).unwrap();
    }

    pub fn name_to_mut_var(&mut self, name: &str) -> &mut Variable {
        let count = self.get_scope_number(name);
        if !self.scopes[count].contains_key(name) {
            let var_type = self.get_member_var_type(name);
            println!("Created new variable '{name}' of type {:?}", var_type);
            self.scopes[count].insert(
                name.to_string(),
                Variable::new(name.to_string(), 0, var_type.clone()),
            );
            self.declare_unknown_global(name, var_type, false)
        }
        return self.scopes[count].get_mut(name).unwrap();
    }

    pub fn set_ownership(&mut self, name: String, has_ownership: bool, span: &span::Span) {
        if name == "NULL" {
            return;
        }
        let variable: &mut Variable = self.name_to_mut_var(&name);
        let var_type: VarType = variable.var_type.clone();

        if let VarType::Owner(type_name, had_ownership) = var_type {
            // Changing an variable's ownership invalidates all its references.
            variable.const_refs.clear();
            variable.mut_refs.clear();
            variable.var_type = VarType::Owner(type_name, has_ownership);

            // Error / Debug prints.
            let (location, _) = get_location_for_offset(self.src, span.start);
            if has_ownership && matches!(self.event_prints, PrintType::Ownership) {
                println!("Made live '{}' on line {}.", name, location.line);
            } else if !has_ownership {
                if !had_ownership {
                    println!(
                        "ERROR: Use of moved value '{}' used on line {}.",
                        name, location.line
                    );
                } else {
                    if matches!(self.event_prints, PrintType::Ownership) {
                        println!("Killed '{}' on line {}.", name, location.line);
                    }
                }
            }
        }
    }

    fn set_all_ownership(&mut self, name: String, has_ownership: bool, span: &span::Span) {
        self.set_ownership(name.clone(), has_ownership, span);
        let period = name.clone() + ".";
        let related: Vec<String> = self.scopes[self.get_scope_number(&name)]
            .keys()
            .filter(|k| k.starts_with(&period))
            .map(|k| k.to_string())
            .collect();
        for member in related {
            self.set_ownership(member, has_ownership, span);
        }
    }

    pub fn get_var_type(
        &mut self,
        declarator: &Declarator,
        specifiers: &Vec<Node<DeclarationSpecifier>>,
    ) -> VarType {
        let mut var_type: VarType = VarType::Copy;
        if !declarator.derived.is_empty()
            && matches!(&declarator.derived[0].node, DerivedDeclarator::Pointer(_))
        {
            // The first derived declarator says this variable is a pointer (Arrays not yet supported).
            var_type = VarType::MutRef(HashSet::new());
            self.previous_struct_name.clear();
            for specifier in specifiers {
                match &specifier.node {
                    DeclarationSpecifier::TypeQualifier(type_qualifier) => {
                        // Const type qualifier (before the type specifier) turns the reference constant.
                        if matches!(&type_qualifier.node, TypeQualifier::Const) {
                            var_type = VarType::ConstRef(HashSet::new());
                        }
                    }
                    DeclarationSpecifier::TypeSpecifier(type_specifier) => {
                        // Once the type specifier is encountered, the reference can no longer be turned const.
                        match &type_specifier.node {
                            TypeSpecifier::Struct(struct_type) => {
                                if let Some(struct_id) = &struct_type.node.identifier {
                                    self.previous_struct_name = struct_id.node.name.clone();
                                }
                            }
                            TypeSpecifier::TypedefName(typedef_id) => {
                                let typedef_name = typedef_id.node.name.clone();
                                if self.structs.contains_key(&typedef_name) {
                                    self.previous_struct_name = typedef_name;
                                }
                            }
                            _ => {}
                        }

                        break;
                    }
                    _ => {}
                }
            }
        } else {
            // Either a copy type or an owner type.
            for specifier in specifiers {
                if let DeclarationSpecifier::TypeSpecifier(type_specifier) = &specifier.node {
                    match &type_specifier.node {
                        TypeSpecifier::Struct(struct_type) => {
                            let Some(identifier) = &struct_type.node.identifier else {
                                break;
                            };
                            let struct_name = identifier.node.name.clone();
                            var_type = VarType::Owner(struct_name, true);
                        }
                        TypeSpecifier::TypedefName(type_identifier) => {
                            let type_name = type_identifier.node.name.clone();
                            if self.structs.contains_key(&type_name) {
                                var_type = VarType::Owner(type_name, true);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        return var_type;
    }

    pub fn get_member_var_type(&mut self, name: &str) -> VarType {
        if !name.contains(".") {
            println!("ISSUE: Unrecognized name '{name}' was not a struct member!");
            return VarType::Copy;
        }
        let final_name = &name[name.rfind('.').unwrap() + 1..];
        let parent_name = &name[..name.rfind('.').unwrap()];
        let parent_type = self.name_to_var(parent_name).var_type.clone();
        if let VarType::Owner(struct_name, _) = parent_type {
            let fields = self
                .structs
                .get(&struct_name.to_string())
                .expect("ISSUE: No struct of specified type");
            return fields
                .get(&final_name.to_string())
                .expect("ISSUE: Parent struct had no matching field!")
                .clone();
        }
        println!("ISSUE: '{parent_name}' is not an owner (struct) type");
        return VarType::Copy;
    }

    pub fn add_struct(&mut self, declaration: &Node<Declaration>) {
        let mut struct_names = HashSet::new();
        let mut struct_members: HashMap<String, VarType> = HashMap::new();
        for specifier in &declaration.node.specifiers {
            let DeclarationSpecifier::TypeSpecifier(type_specifier) = &specifier.node else {
                continue;
            };

            let TypeSpecifier::Struct(struct_type) = &type_specifier.node else {
                continue;
            };

            if let Some(id) = &struct_type.node.identifier {
                struct_names.insert(id.node.name.clone());
            }

            let Some(declarations) = &struct_type.node.declarations else {
                continue;
            };

            // Adding fields to the struct_members mapping from names to VarType.
            for struct_declaration in declarations {
                let StructDeclaration::Field(field) = &struct_declaration.node else {
                    continue;
                };
                for struct_declarator in &field.node.declarators {
                    if let Some(field_declarator) = &struct_declarator.node.declarator {
                        let var_type = self.get_var_type(
                            &field_declarator.node,
                            &self.struct_specifier_to_declaration_specifier(&field.node.specifiers),
                        );
                        if let DeclaratorKind::Identifier(id) = &field_declarator.node.kind.node {
                            struct_members.insert(id.node.name.clone(), var_type);
                        }
                    }
                }
            }
        }

        // Getting any typdef names of this struct.
        for init_declarator in &declaration.node.declarators {
            if let DeclaratorKind::Identifier(id) = &init_declarator.node.declarator.node.kind.node
            {
                struct_names.insert(id.node.name.clone());
            }
        }

        // Adding this struct information under any of its possible names.
        // if !struct_members.is_empty() {
        for name in struct_names {
            self.structs.insert(name, struct_members.clone());
        }
        // }
    }

    // Adds a function declaration to the function mapping to track its parameter types.
    pub fn add_function(
        &mut self,
        declarator: &Node<Declarator>,
        parameter_declarations: &Vec<Node<ParameterDeclaration>>,
    ) {
        let DeclaratorKind::Identifier(function_id) = &declarator.node.kind.node else {
            return;
        };
        let function_name = function_id.node.name.clone();
        let mut function_parameters = Vec::new();
        for parameter_declaration in parameter_declarations {
            let Some(parameter_declarator) = &parameter_declaration.node.declarator else {
                continue;
            };
            // function_parameter is passed as false because that is for adding the variable to the scope of the function bodies we're analyzing.
            let parameter_type = self.get_var_type(
                &parameter_declarator.node,
                &parameter_declaration.node.specifiers,
            );
            function_parameters.push(parameter_type);
        }
        self.functions.insert(function_name, function_parameters);
    }

    // Struct member delcarations use a different set of specifiers than regular declarations.
    pub fn struct_specifier_to_declaration_specifier(
        &self,
        specifiers: &Vec<Node<SpecifierQualifier>>,
    ) -> Vec<Node<DeclarationSpecifier>> {
        let mut out = Vec::new();
        for specifier in specifiers {
            match &specifier.node {
                SpecifierQualifier::TypeSpecifier(ts) => out.push(Node::new(
                    DeclarationSpecifier::TypeSpecifier(ts.clone()),
                    specifier.span,
                )),
                SpecifierQualifier::TypeQualifier(tq) => out.push(Node::new(
                    DeclarationSpecifier::TypeQualifier(tq.clone()),
                    specifier.span,
                )),
                _ => {}
            }
        }
        return out;
    }

    // Adds the variable's name to the proper scope mapping.
    pub fn declare_variable(
        &mut self,
        declarator: &Declarator,
        specifiers: &Vec<Node<DeclarationSpecifier>>,
        function_parameter: bool,
    ) {
        let DeclaratorKind::Identifier(identifier) = &declarator.kind.node else {
            return;
        };

        let name = identifier.node.name.clone();
        let var_type = self.get_var_type(declarator, specifiers);
        let scope: usize = self.scopes.len() - 1;
        self.scopes.last_mut().unwrap().insert(
            name.clone(),
            Variable::new(name.clone(), scope, var_type.clone()),
        );
        self.declare_unknown_global(&name, var_type, function_parameter);
    }

    // Adds a new global for a function parameter pointer to point to (needed for pointers to structs).
    pub fn declare_unknown_global(
        &mut self,
        name: &str,
        var_type: VarType,
        function_parameter: bool,
    ) {
        // Add the "?" unknown variable reference for pointers that are function arguments or struct members.
        if (function_parameter || name.contains("."))
            && matches!(var_type, VarType::ConstRef(_) | VarType::MutRef(_))
        {
            let mut unknown_name = "?".to_string();
            if !self.previous_struct_name.is_empty() {
                // Points to a struct, so it is assumed to point to a unique global of that type.
                unknown_name += &name;
                let unknown_var_type = VarType::Owner(self.previous_struct_name.clone(), true);
                self.scopes[0].insert(
                    unknown_name.to_string(),
                    Variable::new(unknown_name.to_string(), 0, unknown_var_type.clone()),
                );
                self.declare_unknown_global(&unknown_name, unknown_var_type, false)
            } else {
                // Does not point to a struct, so a shared global copy type is used.
                self.scopes[0].insert(
                    "?".to_string(),
                    Variable::new("?".to_string(), 0, VarType::Copy),
                );
            }
            let unknown_id = self.get_id(&unknown_name);

            let new_var = self.name_to_mut_var(&name);

            match &mut new_var.var_type {
                VarType::ConstRef(points_to) => {
                    points_to.insert(unknown_id.clone());
                }
                VarType::MutRef(points_to) => {
                    points_to.insert(unknown_id.clone());
                }
                _ => {}
            }

            let new_id = new_var.id.clone();
            let new_type = new_var.var_type.clone();
            let unknown_var = self.id_to_mut_var(&unknown_id);
            match new_type {
                VarType::ConstRef(_) => unknown_var.const_refs.insert(new_id),
                VarType::MutRef(_) => unknown_var.mut_refs.insert(new_id),
                _ => false,
            };
        }
    }

    // Compiles the name of the current struct member name "x.y.z" into self.member_identifier.
    pub fn get_member_expression_identifier(&mut self, member_expression: &Node<MemberExpression>) {
        self.mute_member_expression = true;
        self.visit_member_expression(&member_expression.node, &member_expression.span);
        self.mute_member_expression = false;
    }

    // Given an expression, sets it to invalid if it is an uncopiable variable.
    pub fn set_expression_is_valid(
        &mut self,
        expression: &Node<Expression>,
        is_valid: bool,
        span: &span::Span,
    ) {
        match &expression.node {
            Expression::Identifier(name) => {
                self.set_all_ownership(name.node.name.clone(), is_valid, span);
            }
            Expression::Member(member_expression) => {
                self.get_member_expression_identifier(member_expression);
                self.set_all_ownership(self.member_identifier.clone(), is_valid, span);
            }
            _ => visit::visit_expression(self, &expression.node, &expression.span),
        }
    }

    pub fn announce_no_ownership(&mut self, name: String, &span: &span::Span) {
        // Creates the middle terms of a struct member identifier in the parent's scope.
        let variable = self.name_to_var(&name);
        if matches!(variable.var_type, VarType::Owner(_, false)) {
            let (location, _) = get_location_for_offset(self.src, span.start);
            println!(
                "ERROR: Use of moved value '{}' used on line {}.",
                name, location.line
            );
        }
    }

    pub fn merge_scopes(&mut self, other_scopes: &Vec<HashMap<String, Variable>>) {
        for i in 0..self.scopes.len() {
            let s = &mut self.scopes[i];

            for (k, v) in other_scopes[i].iter() {
                if let Some(variable) = s.get_mut(k) {
                    // Assume any of the possible references to this variable are all active.
                    variable.const_refs.extend(v.const_refs.clone());
                    variable.mut_refs.extend(v.mut_refs.clone());

                    // Type-specific merging (ownership rounded down, points_to rounded up).
                    match &v.var_type {
                        VarType::Owner(type_name, o1) => {
                            if let VarType::Owner(_, o2) = variable.var_type {
                                variable.var_type = VarType::Owner(type_name.clone(), *o1 && o2);
                            }
                        }
                        VarType::ConstRef(points_to1) | VarType::MutRef(points_to1) => {
                            if let VarType::ConstRef(points_to2) = &mut variable.var_type {
                                points_to2.extend(points_to1.clone());
                            }
                        }
                        VarType::Copy => {}
                    }
                } else {
                    // Inner scopes can create variables in outer scopes (unknown struct members, unknown globals).
                    s.insert(k.clone(), v.clone());
                }
            }
        }
    }

    pub fn print_ownership(&self, &span: &span::Span) {
        let (location, _) = get_location_for_offset(self.src, span.start);
        let out = format!(
            "[{}]",
            self.scopes
                .iter()
                // .skip(1)
                .map(|s| {
                    let inner = s
                        .iter()
                        .map(|(k, v)| {
                            if let VarType::Owner(_, has_ownership) = v.var_type {
                                format!("{k}:{}", has_ownership as i32)
                            } else {
                                k.to_string()
                            }
                        })
                        .intersperse(", ".to_string());
                    format!("{{{}}}", inner.collect::<String>())
                })
                .intersperse("\t".to_string())
                .collect::<String>()
        );
        println!("{}:\t{}", location.line, out);
    }
}

// Reference Functions.
impl<'a> BorrowChecker<'a> {
    // Unlink a ref from all the variables it points to.
    pub fn clear_points_to(&mut self, id: &Id) {
        match &self.id_to_var(id).var_type {
            VarType::ConstRef(points_to) => {
                let ids = points_to.clone();
                for id in ids.iter() {
                    let pointed_to = self.id_to_mut_var(id);
                    pointed_to.const_refs.remove(id);
                }
            }
            VarType::MutRef(points_to) => {
                let ids = points_to.clone();
                for id in ids.iter() {
                    let pointed_to = self.id_to_mut_var(id);
                    pointed_to.mut_refs.remove(id);
                }
            }
            _ => {}
        }

        match &mut self.id_to_mut_var(id).var_type {
            VarType::ConstRef(points_to) => {
                points_to.clear();
            }
            VarType::MutRef(points_to) => {
                points_to.clear();
            }
            _ => {}
        }
    }

    pub fn add_const_ref(&mut self, var_id: &Id, ref_id: &Id) {
        let var = self.id_to_mut_var(var_id);
        var.mut_refs.clear();
        var.const_refs.insert(ref_id.clone());
    }

    pub fn add_mut_ref(&mut self, var_id: &Id, ref_id: &Id) {
        let var = self.id_to_mut_var(var_id);
        var.const_refs.clear();
        var.mut_refs.clear();
        var.mut_refs.insert(ref_id.clone());
    }

    // TODO: struct member names.
    pub fn add_reference(&mut self, lhs: String, expression: &Node<Expression>, _: &span::Span) {
        if let Expression::UnaryOperator(unary_expression) = &expression.node {
            if unary_expression.node.operator.node == UnaryOperator::Address {
                match &unary_expression.node.operand.node {
                    Expression::Identifier(operand) => {
                        let var_id = self.get_id(&operand.node.name);
                        let ref_id = self.get_id(&lhs);

                        match &self.id_to_var(&ref_id).var_type {
                            VarType::ConstRef(_) => {
                                self.clear_points_to(&ref_id);
                                self.add_const_ref(&var_id, &ref_id)
                            }
                            VarType::MutRef(_) => {
                                self.clear_points_to(&ref_id);
                                self.add_mut_ref(&var_id, &ref_id)
                            }
                            _ => {}
                        }

                        match &mut self.id_to_mut_var(&ref_id).var_type {
                            VarType::ConstRef(points_to) => {
                                points_to.insert(var_id.clone());
                            }
                            VarType::MutRef(points_to) => {
                                points_to.insert(var_id.clone());
                            }
                            _ => {}
                        }
                    }
                    Expression::Member(operand) => {
                        self.get_member_expression_identifier(operand);

                        // Borrowing any piece of a struct borrows the entire struct.
                        let parent_name =
                            &self.member_identifier[0..self.member_identifier.find(".").unwrap()];
                        let var_id = self.get_id(parent_name);
                        let ref_id = self.get_id(&lhs);

                        match &self.id_to_var(&ref_id).var_type {
                            VarType::ConstRef(_) => {
                                self.clear_points_to(&ref_id);
                                self.add_const_ref(&var_id, &ref_id)
                            }
                            VarType::MutRef(_) => {
                                self.clear_points_to(&ref_id);
                                self.add_mut_ref(&var_id, &ref_id)
                            }
                            _ => {}
                        }

                        match &mut self.id_to_mut_var(&ref_id).var_type {
                            VarType::ConstRef(points_to) => {
                                points_to.insert(var_id.clone());
                            }
                            VarType::MutRef(points_to) => {
                                points_to.insert(var_id.clone());
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn announce_invalid_reference(&self, name: String, &span: &span::Span) {
        // Creates the middle terms of a struct member identifier in the parent's scope.
        let ref_id = self.get_id(&name);
        let reference = &self.id_to_var(&ref_id).var_type;
        match reference {
            VarType::ConstRef(points_to) => {
                let ids = points_to.clone();
                if ids.is_empty() {
                    let (location, _) = get_location_for_offset(self.src, span.start);
                    println!(
                        "ERROR: using {}, a constant reference to no value, on line {}",
                        ref_id.name, location.line
                    );
                } else {
                    for var_id in ids {
                        if var_id.scope >= self.scopes.len() {
                            let (location, _) = get_location_for_offset(self.src, span.start);
                            println!(
                                "ERROR: using {}, a constant reference to out-of-scope variable {}, on line {}",
                                ref_id.name, var_id.name, location.line
                            );
                        } else {
                            let var = self.id_to_var(&var_id);
                            if !var.const_refs.contains(&ref_id) {
                                let (location, _) = get_location_for_offset(self.src, span.start);
                                println!(
                                "ERROR: using {}, an invalid constant reference to {}, on line {}",
                                ref_id.name, var_id.name, location.line
                            );
                            }
                        }
                    }
                }
            }
            VarType::MutRef(points_to) => {
                let ids = points_to.clone();
                if ids.is_empty() {
                    let (location, _) = get_location_for_offset(self.src, span.start);
                    println!(
                        "ERROR: using {}, a mutable reference to no value, on line {}",
                        ref_id.name, location.line
                    );
                } else {
                    for var_id in ids {
                        if var_id.scope >= self.scopes.len() {
                            let (location, _) = get_location_for_offset(self.src, span.start);
                            println!(
                                "ERROR: using {}, a mutable reference to out-of-scope variable {}, on line {}",
                                ref_id.name, var_id.name, location.line
                            );
                        } else {
                            let var = self.id_to_var(&var_id);
                            if !var.mut_refs.contains(&ref_id) {
                                let (location, _) = get_location_for_offset(self.src, span.start);
                                println!(
                                "ERROR: using {}, an invalid mutable reference to {}, on line {}",
                                ref_id.name, var_id.name, location.line
                            );
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    pub fn print_references(&self, &span: &span::Span) {
        let (location, _) = get_location_for_offset(self.src, span.start);
        let out = format!(
            "[{}]",
            self.scopes
                .iter()
                // .skip(1)
                .map(|s| {
                    let inner = s
                        .iter()
                        .map(|(k, v)| match &v.var_type {
                            VarType::Copy | VarType::Owner(_, _) => {
                                format!(
                                    "{{{}}},{{{}}}'->{}",
                                    v.const_refs
                                        .iter()
                                        .map(|id| id.name.clone())
                                        .intersperse(", ".to_string())
                                        .collect::<String>(),
                                    v.mut_refs
                                        .iter()
                                        .map(|id| id.name.clone())
                                        .intersperse(", ".to_string())
                                        .collect::<String>(),
                                    k
                                )
                            }
                            VarType::ConstRef(points_to) => {
                                format!(
                                    "{k}->{{{}}}",
                                    points_to
                                        .iter()
                                        .map(|id| id.name.clone())
                                        .intersperse(", ".to_string())
                                        .collect::<String>()
                                )
                            }
                            VarType::MutRef(points_to) => {
                                format!(
                                    "{k}'->{{{}}}",
                                    points_to
                                        .iter()
                                        .map(|id| id.name.clone())
                                        .intersperse(", ".to_string())
                                        .collect::<String>()
                                )
                            }
                        })
                        .intersperse("; ".to_string());
                    format!("{{{}}}", inner.collect::<String>())
                })
                .intersperse("\t".to_string())
                .collect::<String>()
        );
        println!("{}:\t{}", location.line, out);
    }
}
