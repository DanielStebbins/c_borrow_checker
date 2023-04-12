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
    - Replace kill_next with not traversing those nodes.
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

struct Reference {
    identifier: String,
}

struct BorrowChecker<'a> {
    src: &'a str,
    dead: HashSet<String>,
    mutable_references: HashMap<String, String>,
    mute_member_expression: bool,
    member_count: u32,
    member_identifier_pieces: Vec<String>,
    member_identifier: String,
    set_prints: bool,
    ownership_debug_prints: bool,
    reference_debug_prints: bool,
}

// Functions that mutate and print information about the dead variables.
impl<'a> BorrowChecker<'a> {
    fn kill(&mut self, identifier: String, &span: &span::Span) {
        if identifier == "NULL" {
            return;
        }
        let (location, _) = get_location_for_offset(self.src, span.start);
        if !self.dead.insert(identifier.clone()) {
            println!(
                "ERROR: Dead identifier '{}' used on line {}.",
                identifier, location.line
            );
        } else {
            if self.ownership_debug_prints {
                println!(
                    "Killed identifier '{}' on line {}.",
                    identifier, location.line
                );
            }
        }
    }

    // Given an expression, kills it if it is an identifier.
    // TODO: Expand this to member identifiers.
    fn kill_if_identifier(&mut self, expression: &Node<Expression>, span: &span::Span) {
        match &expression.node {
            Expression::Identifier(identifier) => {
                self.kill(identifier.node.name.clone(), span);
            }
            Expression::Member(member_expression) => {
                self.get_member_expression_identifier(member_expression);
                self.kill(self.member_identifier.clone(), span);
            }
            _ => visit::visit_expression(self, &expression.node, &expression.span),
        }
    }

    fn make_live(&mut self, identifier: String, &span: &span::Span) {
        // Remove the identifier itself.
        self.dead.remove(&identifier);

        // Remove all identifiers that start with the given identifier in a member "." chain.
        let period = identifier.clone() + ".";
        let arrow = identifier.clone() + "->";
        self.dead
            .retain(|x| !x.starts_with(&period) && !x.starts_with(&arrow));

        if self.ownership_debug_prints {
            let (location, _) = get_location_for_offset(self.src, span.start);
            println!(
                "Made live identifier '{}' on line {}.",
                identifier, location.line
            );
        }
    }

    // Given an expression, make it live if it is an identifier.
    // TODO: Expand this to member identifiers.
    fn make_live_if_identifier(&mut self, expression: &Node<Expression>, span: &span::Span) {
        match &expression.node {
            Expression::Identifier(identifier) => {
                self.make_live(identifier.node.name.clone(), span);
            }
            Expression::Member(member_expression) => {
                self.get_member_expression_identifier(member_expression);
                self.make_live(self.member_identifier.clone(), span);
            }
            _ => visit::visit_expression(self, &expression.node, &expression.span),
        }
    }

    fn get_member_expression_identifier(&mut self, member_expression: &Node<MemberExpression>) {
        self.mute_member_expression = true;
        self.visit_member_expression(&member_expression.node, &member_expression.span);
        self.mute_member_expression = false;
    }

    fn announce_if_dead(&self, identifier: String, &span: &span::Span) {
        if self.dead.contains(&identifier) {
            let (location, _) = get_location_for_offset(self.src, span.start);
            println!(
                "ERROR: Dead identifier '{}' used on line {}.",
                identifier, location.line
            );
        }
    }

    fn print_dead_pre(&self, &span: &span::Span) {
        let (location, _) = get_location_for_offset(self.src, span.start);
        println!("{}:\t{:?}", location.line, self.dead);
    }

    fn print_dead_post(&self) {
        println!("\t{:?}", self.dead);
    }
}

// Reference checking functions.
impl<'a> BorrowChecker<'a> {
    fn add_if_reference(&mut self, lhs: String, expression: &Node<Expression>, span: &span::Span) {
        if let Expression::UnaryOperator(unary_expression) = &expression.node {
            if let Expression::Identifier(operand) = &unary_expression.node.operand.node {
                if unary_expression.node.operator.node == UnaryOperator::Address {
                    self.mutable_references
                        .insert(operand.node.name.clone(), lhs);
                }
            }
        }
    }

    fn announce_if_mutable(&self, identifier: &String, &span: &span::Span) -> bool {
        if self.mutable_references.contains_key(identifier) {
            let (location, _) = get_location_for_offset(self.src, span.start);
            println!(
                "ERROR: Trying to create a second reference to '{}' on line {}. A mutable reference to '{}' already exists.",
                identifier, location.line, identifier
            );
            true
        } else {
            false
        }
    }

    fn print_mut_references_pre(&self, &span: &span::Span) {
        let (location, _) = get_location_for_offset(self.src, span.start);
        println!("{}:\t{:?}", location.line, self.mutable_references);
    }

    fn print_mut_references_post(&self) {
        println!("\t{:?}", self.mutable_references);
    }
}

impl<'ast, 'a> visit::Visit<'ast> for BorrowChecker<'a> {
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
                    self.kill_if_identifier(&expression, span);
                }
                _ => visit::visit_initializer(self, &initializer.node, span),
            }
        }

        // LHS
        if let DeclaratorKind::Identifier(identifier) = &init_declarator.declarator.node.kind.node {
            self.make_live(identifier.node.name.clone(), span);

            // Possibly adding a reference, which requires the LHS's identifier.
            if let Some(ref initializer) = init_declarator.initializer {
                if let Initializer::Expression(expression) = &initializer.node {
                    self.add_if_reference(identifier.node.name.clone(), &expression, span);
                }
            }
        }
    }

    fn visit_binary_operator_expression(
        &mut self,
        boe: &'ast ast::BinaryOperatorExpression,
        span: &'ast span::Span,
    ) {
        // Visit the RHS first to check its expressions.
        if boe.operator.node != BinaryOperator::Assign {
            visit::visit_binary_operator_expression(self, boe, span);
        } else {
            self.kill_if_identifier(&boe.rhs, span);
            self.make_live_if_identifier(&boe.lhs, span);
        }
    }

    // Parameters passed to a function are made live.
    fn visit_parameter_declaration(
        &mut self,
        parameter_declaration: &'ast ParameterDeclaration,
        span: &'ast span::Span,
    ) {
        if let Some(declarator) = &parameter_declaration.declarator {
            if let DeclaratorKind::Identifier(identifier) = &declarator.node.kind.node {
                self.make_live(identifier.node.name.clone(), span)
            }
        }
    }

    fn visit_call_expression(
        &mut self,
        call_expression: &'ast CallExpression,
        span: &'ast span::Span,
    ) {
        visit::visit_expression(
            self,
            &call_expression.callee.node,
            &call_expression.callee.span,
        );

        for argument in &call_expression.arguments {
            self.kill_if_identifier(argument, span);
        }
    }

    // =============================================== Small Expressions ===============================================

    fn visit_identifier(&mut self, identifier: &'ast Identifier, span: &'ast span::Span) {
        if self.member_count > 0 {
            self.member_identifier_pieces.push(identifier.name.clone());
            self.announce_if_dead(self.member_identifier_pieces.join("."), span);
        } else {
            self.announce_if_dead(identifier.name.clone(), span);
        }
    }

    fn visit_member_expression(
        &mut self,
        member_expression: &'ast MemberExpression,
        span: &'ast span::Span,
    ) {
        // Compiling a member identifier (struct_name.x.y ..., if struct_name.x is dead, it's an error).
        // This is the recursive part.
        self.member_count += 1;
        self.visit_expression(
            &member_expression.expression.node,
            &member_expression.expression.span,
        );
        self.member_count -= 1;

        self.member_identifier_pieces
            .push(member_expression.identifier.node.name.clone());

        if self.member_count > 0 || !self.mute_member_expression {
            self.announce_if_dead(self.member_identifier_pieces.join("."), span);
        }
        if self.member_count == 0 {
            self.member_identifier = self.member_identifier_pieces.join(".");
            self.member_identifier_pieces.clear();
        }
    }

    // =============================================== References ===============================================

    // References are not killed (&x does not kill x).
    fn visit_unary_operator_expression(
        &mut self,
        uoe: &'ast UnaryOperatorExpression,
        span: &'ast span::Span,
    ) {
        if let Expression::Identifier(operand) = &uoe.operand.node {
            if uoe.operator.node == UnaryOperator::Address {
                _ = self.announce_if_mutable(&operand.node.name, span)
            }
        }
        visit::visit_unary_operator_expression(self, uoe, span);
    }

    // =============================================== Control Flow ===============================================
    // This union logic might be better any time moving into a new statement, check the tree.
    fn visit_if_statement(&mut self, if_statement: &'ast IfStatement, _: &'ast span::Span) {
        self.visit_expression(&if_statement.condition.node, &if_statement.condition.span);

        let temp = self.dead.clone();
        self.visit_statement(
            &if_statement.then_statement.node,
            &if_statement.then_statement.span,
        );
        if let Some(ref else_statement) = if_statement.else_statement {
            let if_dead = self.dead.clone();
            self.dead = temp.clone();
            self.visit_statement(&else_statement.node, &else_statement.span);
            self.dead.extend(temp);
            self.dead.extend(if_dead);
        } else {
            self.dead.extend(temp);
        }
    }

    // =============================================== Debug ===============================================
    // For printing the dead set each "line".
    fn visit_block_item(&mut self, block_item: &'ast BlockItem, span: &'ast span::Span) {
        if self.set_prints {
            // self.print_dead_pre(span);
            self.print_mut_references_pre(span);
        }
        visit::visit_block_item(self, block_item, span);
        if self.set_prints {
            // self.print_dead_post();
            self.print_mut_references_post();
        }
    }
}

fn main() {
    let file_path = "inputs\\borrow2.c";
    let config = Config::default();
    let result = parse(&config, file_path);

    let parse = result.expect("Parsing Error!\n");

    let mut ownership_checker = BorrowChecker {
        src: &parse.source,
        dead: HashSet::new(),
        mutable_references: HashMap::new(),
        mute_member_expression: false,
        member_count: 0,
        member_identifier_pieces: Vec::new(),
        member_identifier: "".to_string(),
        set_prints: false,
        ownership_debug_prints: false,
        reference_debug_prints: false,
    };

    if ownership_checker.ownership_debug_prints || ownership_checker.reference_debug_prints {
        let s = &mut String::new();
        let mut printer = Printer::new(s);
        printer.visit_translation_unit(&parse.unit);
        println!("{s}");
    }

    ownership_checker.visit_translation_unit(&parse.unit);
}

// RUN                         cargo clippy            to view
// git commit -m ""     ->     cargo clippy --fix      to fix
