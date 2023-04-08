/*
Rules:
    - Assigning to a variable makes it un-dead.
    - Using a variable in any way not marked with 'self.kill_next = false;' makes the variable lose ownership.
    - Struct members are killed all together: 'struct.value.x'. If any piece 'struct.value' from left to right is dead, it is announced.
*/

/*
Ranting:
    - I can't know every field of every struct in an arbitrary program, so it must be possible to kill a label the first time I see it, without it ever having been alive.
    - Avoid unwrap() when possible
    - x = x     with dead x reports the error, but re-lives x anyways.
*/

/*
TODO:
    - { } scope levels.
    - Variable name shadowing.
    - Replace kill_next with not traversing those nodes.
*/

use lang_c::ast::*;
use lang_c::driver::*;
use lang_c::loc::get_location_for_offset;
use lang_c::print::*;
use lang_c::visit::*;
use lang_c::*;
use std::collections::HashSet;

struct OwnershipChecker<'a> {
    src: &'a str,
    assignments: i32,
    dead: HashSet<String>,
    kill_next: bool,
    live_member: bool,
    member_count: u32,
    member_identifier: Vec<String>,
}

// Functions that mutate and print information about the dead variables.
impl<'a> OwnershipChecker<'a> {
    fn kill(&mut self, identifier: String, &span: &span::Span) {
        let (location, _) = get_location_for_offset(self.src, span.start);
        if !self.dead.insert(identifier.clone()) {
            println!(
                "Dead identifier '{}' used on line {}.",
                identifier, location.line
            );
        } else {
            println!(
                "Killed identifier '{}' on line {}.",
                identifier, location.line
            );
            println!("{:?}", self.dead);
        }
    }

    fn announce_if_dead(&self, identifier: String, &span: &span::Span) {
        if self.dead.contains(&identifier) {
            let (location, _) = get_location_for_offset(self.src, span.start);
            println!(
                "Dead identifier '{}' used on line {}.",
                identifier, location.line
            );
        }
    }

    fn make_live(&mut self, identifier: String, &span: &span::Span) {
        self.dead.remove(&identifier);
        // self.kill_next = false;

        let (location, _) = get_location_for_offset(self.src, span.start);
        println!(
            "Live-d identifier '{}' on line {}.",
            identifier, location.line
        );
    }
}

impl<'ast, 'a> visit::Visit<'ast> for OwnershipChecker<'a> {
    // =============================================== Assignments (make_live) ===============================================
    // Make identifiers valid if they are on the LHS of an assignment or are declared.
    fn visit_init_declarator(
        &mut self,
        init_declarator: &'ast InitDeclarator,
        span: &'ast span::Span,
    ) {
        if let Some(ref initializer) = init_declarator.initializer {
            visit::visit_initializer(self, &initializer.node, &initializer.span);
        }
        self.assignments += 1;
        if let DeclaratorKind::Identifier(id) = &init_declarator.declarator.node.kind.node {
            self.dead.remove(&id.node.name);
            // self.kill_next = false;
            let (location, _) = get_location_for_offset(self.src, span.start);
            println!(
                "Live-d identifier '{}' on line {}.",
                id.node.name, location.line
            );
        }
    }

    fn visit_binary_operator_expression(
        &mut self,
        boe: &'ast ast::BinaryOperatorExpression,
        span: &'ast span::Span,
    ) {
        // Visit the RHS first to check its expressions.
        self.visit_expression(&boe.rhs.node, &boe.rhs.span);
        self.visit_binary_operator(&boe.operator.node, &boe.operator.span);

        // Make the assigned-to LHS valid.
        if boe.operator.node == BinaryOperator::Assign {
            self.assignments += 1;
            match &boe.lhs.node {
                Expression::Identifier(lhs) => {
                    self.make_live(lhs.node.name.clone(), span);
                }
                Expression::Member(lhs) => {
                    self.live_member = true;
                    self.visit_member_expression(&lhs.node, span);
                }
                _ => (),
            }
        } else {
            // Don't visit the LHS of assignments, that's all handled above.
            self.visit_expression(&boe.lhs.node, &boe.lhs.span);
        }
    }

    // =============================================== Small Expressions ===============================================
    // Kill any identifier that is used in a context not on the accepted list. (references, function definitions, function calls).
    fn visit_identifier(&mut self, identifier: &'ast Identifier, span: &'ast span::Span) {
        // We're making a member identifier (struct_name.x.y ..., no need to kill x and y, but if struct_name.x is dead, it's an error).
        if self.member_count > 0 {
            self.member_identifier.push(identifier.name.clone());
            self.announce_if_dead(self.member_identifier.join("."), span);
        } else if self.kill_next {
            self.kill(identifier.name.clone(), span);
        }
        self.kill_next = true;
        visit::visit_identifier(self, identifier, span)
    }

    fn visit_member_expression(
        &mut self,
        member_expression: &'ast MemberExpression,
        span: &'ast span::Span,
    ) {
        self.member_count += 1;
        visit::visit_member_expression(self, member_expression, span);
        self.member_count -= 1;

        if self.member_count == 0 && !self.member_identifier.is_empty() {
            let identifier = self.member_identifier.join(".");
            self.member_identifier.clear();

            if self.live_member {
                self.make_live(identifier, span);
                self.live_member = false;
            } else if self.kill_next {
                self.kill(identifier, span);
            }
            self.kill_next = true;
        }
    }

    // =============================================== No Kills ===============================================
    // References are not killed (&x does not kill x).
    fn visit_unary_operator_expression(
        &mut self,
        uoe: &'ast UnaryOperatorExpression,
        span: &'ast span::Span,
    ) {
        if uoe.operator.node == UnaryOperator::Address {
            self.kill_next = false;
        }
        visit::visit_unary_operator_expression(self, uoe, span)
    }

    // Function calls are not killed (foo(x) does not kill foo).
    fn visit_call_expression(
        &mut self,
        call_expression: &'ast CallExpression,
        span: &'ast span::Span,
    ) {
        self.kill_next = false;
        visit::visit_call_expression(self, call_expression, span)
    }

    fn visit_function_definition(
        &mut self,
        function_definition: &'ast FunctionDefinition,
        span: &'ast span::Span,
    ) {
        self.kill_next = false;
        visit::visit_function_definition(self, function_definition, span)
    }

    // Don't kill the struct identifier in "struct_name x;"
    fn visit_type_specifier(
        &mut self,
        type_specifier: &'ast TypeSpecifier,
        span: &'ast span::Span,
    ) {
        if let TypeSpecifier::TypedefName(_) = &type_specifier {
            self.kill_next = false;
        }
        visit::visit_type_specifier(self, type_specifier, span)
    }

    // Don't kill a struct name during its definition (typdef struct struct_name { ... )
    fn visit_struct_type(&mut self, struct_type: &'ast StructType, span: &'ast span::Span) {
        if struct_type.identifier.is_some() {
            self.kill_next = false;
        }
        visit::visit_struct_type(self, struct_type, span)
    }

    // Don't kill struct members being declared.
    fn visit_struct_field(&mut self, struct_field: &'ast StructField, span: &'ast span::Span) {
        self.kill_next = false;
        visit::visit_struct_field(self, struct_field, span)
    }

    // =============================================== Debug ===============================================
    // For printing the dead set each "line".
    fn visit_block_item(&mut self, block_item: &'ast BlockItem, span: &'ast span::Span) {
        println!("{:?}", self.dead);
        visit::visit_block_item(self, block_item, span)
    }
}

fn main() {
    let file_path = "inputs\\ownership2.c";
    let config = Config::default();
    let result = parse(&config, file_path);

    let parse = result.expect("Parsing Error!\n");

    let mut ownership_checker = OwnershipChecker {
        src: &parse.source,
        assignments: 0,
        dead: HashSet::new(),
        kill_next: true,
        live_member: false,
        member_count: 0,
        member_identifier: Vec::new(),
    };

    let s = &mut String::new();
    let mut printer = Printer::new(s);
    printer.visit_translation_unit(&parse.unit);
    println!("{s}");

    ownership_checker.visit_translation_unit(&parse.unit);
    println!("Dead variables at exit: {:?}", ownership_checker.dead);
}

// RUN                         cargo clippy            to view
// git commit -m ""     ->     cargo clippy --fix      to fix
