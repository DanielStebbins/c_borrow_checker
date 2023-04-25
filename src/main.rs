/*
Rules:
    - Assigning to a variable makes it un-dead.
    - Using a variable alone on the RHS of an assignment or as an argument to a function call makes it dead.
    - Struct members are killed all together: 'struct.value.x'. If any piece 'struct.value' from left to right is dead, it is announced.
    - If statements make copies of the dead variables state. At the end of the if/else, all the sets are unioned together.
    - Any use of a variable checks whether that variable has ownership (is not dead). If it is dead, an error is printed.
    - Any &x triggers a check to see if x already has a mutable reference. If it does, an error is printed.
*/

/*
Ranting:
    - I can't know every field of every struct in an arbitrary program, so it must be possible to kill a label the first time I see it, without it ever having been alive.
    - Avoid unwrap() when possible
    - x = x     with dead x reports the error, but re-lives x anyways.
*/

/*
Limitations:
    - No library imports in the input. The checker will try to analyze the whole library. This means it's unaware of library function signatures.
    - To get line-by-line prints, each block (if, for, while, ...) must have {}
    - No &&x, only &x are recognized as references because they are immeditately followed by an identifier.
*/

/*
TODO:
    - { } scope levels.
    - Variable name shadowing (init_declarator inside block {} should not un-kill higher-scope dead variable, killing low scopre variable should have no effect on outer scope variable).
    - Support += (AssignPlus) and other shorthand.
    - Count . and -> separately.
*/

use lang_c::ast::*;
use lang_c::driver::*;
use lang_c::loc::*;
use lang_c::print::*;
use lang_c::span::*;
use lang_c::visit::*;
use lang_c::*;
use std::collections::HashMap;
use std::collections::HashSet;

enum PrintType {
    Ownership,
    Reference,
    ErrorOnly,
}

struct Id {
    name: String,
    scope: usize,
}

struct Variable {
    id: Id,
    is_valid: bool,
    is_copy_type: bool,
    const_refs: HashSet<Id>,
    mut_ref: Option<Id>,
    points_to: Option<Id>,
}

impl Variable {
    fn new(name: String, scope: usize) -> Self {
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

struct BorrowChecker<'a> {
    src: &'a str,
    scopes: Vec<HashMap<String, Variable>>,

    mute_member_expression: bool,
    member_count: u32,
    member_identifier_pieces: Vec<String>,
    member_identifier: String,

    set_prints: PrintType,
    event_prints: PrintType,
}

// Functions that mutate and print information about the dead variables.
impl<'a> BorrowChecker<'a> {
    // Finds the most local (highest count) scope where the given name exists.
    fn get_scope_number(&self, name: String) -> usize {
        let mut count: usize = self.scopes.len() - 1;
        for scope in self.scopes.iter().rev() {
            if scope.contains_key(&name) {
                return count;
            }
            count -= 1;
        }
        return count;
    }

    fn get_variable(&mut self, name: &str) -> &mut Variable {
        let mut count = 0;
        for scope in 0..self.scopes.len() {
            if self.scopes[scope].contains_key(name) {
                count = scope;
            }
        }
        if !self.scopes[count].contains_key(name) {
            self.scopes[count].insert(name.to_string(), Variable::new(name.to_string(), 0));
        }
        return self.scopes[count].get_mut(name).unwrap();
    }

    fn set_is_valid(&mut self, name: String, is_valid: bool, span: &span::Span) {
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

    fn declare_variable(&mut self, name: String) {
        let scope: usize = self.scopes.len() - 1;
        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.clone(), Variable::new(name, scope));
    }

    // Given an expression, sets it to invalid it if it is an uncopiable variable.
    // TODO: Make work with struct types again.
    fn unowner(&mut self, expression: &Node<Expression>, span: &span::Span) {
        match &expression.node {
            Expression::Identifier(name) => {
                self.set_is_valid(name.node.name.clone(), false, span);
            }
            // Expression::Member(member_expression) => {
            //     self.get_member_expression_identifier(member_expression);
            //     self.kill(self.member_identifier.clone(), span);
            // }
            _ => visit::visit_expression(self, &expression.node, &expression.span),
        }
    }

    fn print_ownership(&self, &span: &span::Span) {
        let (location, _) = get_location_for_offset(self.src, span.start);
        let mut outer = Vec::new();
        for scope in &self.scopes {
            let mut inner = Vec::new();
            for key in scope.keys() {
                let mut current = key.to_string();
                if !scope.get(key).unwrap().is_copy_type {
                    current.push(':');
                    if scope.get(key).unwrap().is_valid {
                        current.push('1')
                    } else {
                        current.push('0')
                    }
                }
                inner.push(current);
            }
            let mut temp = "{".to_string();
            temp.push_str(&inner.join(", "));
            temp.push('}');
            outer.push(temp);
        }
        let mut out = "[".to_string();
        out.push_str(&outer.join("\t"));
        out.push(']');
        println!("{}:\t{}", location.line, out);
    }

    // TODO
    fn print_references(&self, &span: &span::Span) {
        let (location, _) = get_location_for_offset(self.src, span.start);
        let out = "TODO: PRINT REFERENCES";
        println!("{}:\t{:?}", location.line, out);
    }
}

impl<'ast, 'a> visit::Visit<'ast> for BorrowChecker<'a> {
    // Triggers scope changes.
    fn visit_statement(&mut self, statement: &'ast Statement, span: &'ast span::Span) {
        // Add a new scope layer for this block.
        if let Statement::Compound(_) = statement {
            self.scopes.push(HashMap::new());
        }

        // Run the block.
        visit::visit_statement(self, statement, span);

        // Remove the block's scope layer.
        if let Statement::Compound(_) = statement {
            self.scopes.pop();
        }
    }

    // =============================================== Assignments (make_live) ===============================================
    // Make identifiers valid if they are on the LHS of an assignment or are declared.
    fn visit_init_declarator(
        &mut self,
        init_declarator: &'ast InitDeclarator,
        span: &'ast span::Span,
    ) {
        // RHS
        if let Some(ref initializer) = init_declarator.initializer {
            match &initializer.node {
                Initializer::Expression(expression) => {
                    self.unowner(&expression, span);
                }
                _ => visit::visit_initializer(self, &initializer.node, span),
            }
        }

        // LHS
        if let DeclaratorKind::Identifier(identifier) = &init_declarator.declarator.node.kind.node {
            self.declare_variable(identifier.node.name.clone());

            // Possibly adding a reference, which requires the LHS's identifier.
            // if let Some(ref initializer) = init_declarator.initializer {
            //     if let Initializer::Expression(expression) = &initializer.node {
            //         self.add_if_reference(identifier.node.name.clone(), &expression, span);
            //     }
            // }
        }
    }

    fn visit_binary_operator_expression(
        &mut self,
        boe: &'ast ast::BinaryOperatorExpression,
        span: &'ast span::Span,
    ) {
        if boe.operator.node != BinaryOperator::Assign {
            visit::visit_binary_operator_expression(self, boe, span);
        } else {
            self.unowner(&boe.rhs, span);
            match &boe.lhs.node {
                Expression::Identifier(identifier) => {
                    self.set_is_valid(identifier.node.name.clone(), true, span);
                    // self.add_if_reference(identifier.node.name.clone(), &boe.rhs, span)
                }
                // Expression::Member(member_expression) => {
                //     self.get_member_expression_identifier(member_expression);
                //     self.make_live(self.member_identifier.clone(), span);
                //     self.add_if_reference(self.member_identifier.clone(), &boe.rhs, span)
                // }
                _ => visit::visit_expression(self, &boe.lhs.node, &boe.lhs.span),
            }
        }
    }

    // // Parameters passed to a function are made live.
    // fn visit_parameter_declaration(
    //     &mut self,
    //     parameter_declaration: &'ast ParameterDeclaration,
    //     span: &'ast span::Span,
    // ) {
    //     if let Some(declarator) = &parameter_declaration.declarator {
    //         if let DeclaratorKind::Identifier(identifier) = &declarator.node.kind.node {
    //             self.make_live(identifier.node.name.clone(), span)
    //         }
    //     }
    // }

    // fn visit_call_expression(
    //     &mut self,
    //     call_expression: &'ast CallExpression,
    //     span: &'ast span::Span,
    // ) {
    //     visit::visit_expression(
    //         self,
    //         &call_expression.callee.node,
    //         &call_expression.callee.span,
    //     );

    //     for argument in &call_expression.arguments {
    //         self.kill_if_identifier(argument, span);
    //     }
    // }

    // // =============================================== Small Expressions ===============================================

    // fn visit_identifier(&mut self, identifier: &'ast Identifier, span: &'ast span::Span) {
    //     if self.member_count > 0 {
    //         self.member_identifier_pieces.push(identifier.name.clone());
    //         self.announce_if_dead(self.member_identifier_pieces.join("."), span);
    //     } else {
    //         self.announce_if_dead(identifier.name.clone(), span);
    //     }
    // }

    // fn visit_member_expression(
    //     &mut self,
    //     member_expression: &'ast MemberExpression,
    //     span: &'ast span::Span,
    // ) {
    //     // Compiling a member identifier (struct_name.x.y ..., if struct_name.x is dead, it's an error).
    //     // This is the recursive part.
    //     self.member_count += 1;
    //     self.visit_expression(
    //         &member_expression.expression.node,
    //         &member_expression.expression.span,
    //     );
    //     self.member_count -= 1;

    //     self.member_identifier_pieces
    //         .push(member_expression.identifier.node.name.clone());

    //     if self.member_count > 0 || !self.mute_member_expression {
    //         self.announce_if_dead(self.member_identifier_pieces.join("."), span);
    //     }
    //     if self.member_count == 0 {
    //         self.member_identifier = self.member_identifier_pieces.join(".");
    //         self.member_identifier_pieces.clear();
    //     }
    // }

    // // =============================================== References ===============================================

    // // References are not killed (&x does not kill x).
    // fn visit_unary_operator_expression(
    //     &mut self,
    //     uoe: &'ast UnaryOperatorExpression,
    //     span: &'ast span::Span,
    // ) {
    //     if let Expression::Identifier(operand) = &uoe.operand.node {
    //         if uoe.operator.node == UnaryOperator::Address {
    //             self.announce_if_new_ref_and_mutable(&operand.node.name, span)
    //         }
    //     }
    //     visit::visit_unary_operator_expression(self, uoe, span);
    // }

    // // =============================================== Control Flow ===============================================
    // // This union logic might be better any time moving into a new statement, check the tree.
    // fn visit_if_statement(&mut self, if_statement: &'ast IfStatement, _: &'ast span::Span) {
    //     self.visit_expression(&if_statement.condition.node, &if_statement.condition.span);

    //     let temp = self.dead.clone();
    //     self.visit_statement(
    //         &if_statement.then_statement.node,
    //         &if_statement.then_statement.span,
    //     );
    //     if let Some(ref else_statement) = if_statement.else_statement {
    //         let if_dead = self.dead.clone();
    //         self.dead = temp.clone();
    //         self.visit_statement(&else_statement.node, &else_statement.span);
    //         self.dead.extend(temp);
    //         self.dead.extend(if_dead);
    //     } else {
    //         self.dead.extend(temp);
    //     }
    // }

    // =============================================== Debug ===============================================
    // For printing the dead set each "line".
    fn visit_block_item(&mut self, block_item: &'ast BlockItem, span: &'ast span::Span) {
        match self.set_prints {
            PrintType::Ownership => self.print_ownership(span),
            PrintType::Reference => self.print_references(span),
            PrintType::ErrorOnly => {}
        }
        visit::visit_block_item(self, block_item, span);
        match self.set_prints {
            PrintType::Ownership => self.print_ownership(span),
            PrintType::Reference => self.print_references(span),
            PrintType::ErrorOnly => {}
        }
    }
}

fn main() {
    let file_path = "inputs\\ownership0.c";
    let config = Config::default();
    let result = parse(&config, file_path);

    let parse = result.expect("Parsing Error!\n");

    let mut ownership_checker = BorrowChecker {
        src: &parse.source,
        scopes: vec![HashMap::new()],

        mute_member_expression: false,
        member_count: 0,
        member_identifier_pieces: Vec::new(),
        member_identifier: "".to_string(),

        set_prints: PrintType::Ownership,
        event_prints: PrintType::ErrorOnly,
    };

    let s = &mut String::new();
    let mut printer = Printer::new(s);
    printer.visit_translation_unit(&parse.unit);
    println!("{s}");

    ownership_checker.visit_translation_unit(&parse.unit);
}

// RUN                         cargo clippy            to view
// git commit -m ""     ->     cargo clippy --fix      to fix
