use crate::variable::Variable;
use lang_c::ast::*;
use lang_c::loc::*;
use lang_c::span::*;
use lang_c::visit::Visit;
use lang_c::*;
use std::borrow::Borrow;
use std::collections::HashMap;

pub enum PrintType {
    Ownership,
    Reference,
    ErrorOnly,
}

pub struct BorrowChecker<'a> {
    pub src: &'a str,
    pub scopes: Vec<HashMap<String, Variable>>,

    // Struct member identifier compilation.
    pub mute_member_expression: bool,
    pub member_count: u32,
    pub member_identifier_pieces: Vec<String>,
    pub member_identifier: String,

    pub set_prints: PrintType,
    pub event_prints: PrintType,
}

// Functions that mutate and print information about the dead variables.
impl<'a> BorrowChecker<'a> {
    // Finds the most local (highest count) scope where the given name exists.
    pub fn get_scope_number(&self, mut name: &str) -> usize {
        let mut count: usize = self.scopes.len() - 1;
        if name.contains(".") {
            name = &name[..name.find(".").unwrap()];
        } else if name.contains("->") {
            name = &name[..name.find("->").unwrap()];
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

    pub fn get_variable(&mut self, name: &str) -> &mut Variable {
        let count = self.get_scope_number(name);
        if !self.scopes[count].contains_key(name) {
            self.scopes[count].insert(name.to_string(), Variable::new(name.to_string(), 0));
        }
        return self.scopes[count].get_mut(name).unwrap();
    }

    pub fn set_is_valid(&mut self, name: String, is_valid: bool, span: &span::Span) {
        if name == "NULL" {
            return;
        }
        let (location, _) = get_location_for_offset(self.src, span.start);
        let variable: &mut Variable = self.get_variable(&name);
        let was_valid: bool = variable.is_valid;
        let was_copy_type: bool = variable.is_copy_type;
        variable.is_valid = is_valid;

        // Error / Debug prints.
        // self.announce_if_ref_to_moved(&name, span);
        if is_valid && matches!(self.event_prints, PrintType::Ownership) {
            println!("Made live '{}' on line {}.", name, location.line);
        } else if !is_valid && !was_copy_type {
            if !was_valid {
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

    pub fn declare_variable(&mut self, name: String) {
        let scope: usize = self.scopes.len() - 1;
        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.clone(), Variable::new(name, scope));
    }

    fn get_member_expression_identifier(&mut self, member_expression: &Node<MemberExpression>) {
        self.mute_member_expression = true;
        self.visit_member_expression(&member_expression.node, &member_expression.span);
        self.mute_member_expression = false;
    }

    // Given an expression, sets it to invalid it if it is an uncopiable variable.
    // TODO: Make work with struct types again.
    pub fn set_expression_is_valid(
        &mut self,
        expression: &Node<Expression>,
        is_valid: bool,
        span: &span::Span,
    ) {
        match &expression.node {
            Expression::Identifier(name) => {
                self.set_is_valid(name.node.name.clone(), is_valid, span);
            }
            Expression::Member(member_expression) => {
                self.get_member_expression_identifier(member_expression);
                self.set_is_valid(self.member_identifier.clone(), is_valid, span);
            }
            _ => visit::visit_expression(self, &expression.node, &expression.span),
        }
    }

    pub fn announce_if_dead(&mut self, name: String, &span: &span::Span) {
        // Creates the middle terms of a struct member identifier in the parent's scope.
        let variable = self.get_variable(&name);
        if !variable.is_valid {
            let (location, _) = get_location_for_offset(self.src, span.start);
            println!(
                "ERROR: Dead identifier '{}' used on line {}.",
                name, location.line
            );
        }
    }

    pub fn merge_scopes(&mut self, other_scopes: &Vec<HashMap<String, Variable>>) {
        for i in 0..self.scopes.len() {
            let s = &mut self.scopes[i];
            for (k, v) in other_scopes[i].iter() {
                if let Some(variable) = s.get_mut(k) {
                    variable.is_valid = variable.is_valid && v.is_valid;
                } else {
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
                .skip(1)
                .map(|s| {
                    let inner = s
                        .iter()
                        .map(|(k, v)| {
                            if !v.is_copy_type {
                                format!("{k}:{}", v.is_valid as i32)
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

    // TODO
    pub fn print_references(&self, &span: &span::Span) {
        let (location, _) = get_location_for_offset(self.src, span.start);
        let out = "TODO: PRINT REFERENCES";
        println!("{}:\t{:?}", location.line, out);
    }
}
