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
Limitations:
    - No library imports in the input. The checker will try to analyze the whole library. This means it's unaware of library function signatures.
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

    debug_prints: bool,
}

// Functions that mutate and print information about the dead variables.
impl<'a> OwnershipChecker<'a> {
    fn kill(&mut self, identifier: String, &span: &span::Span) {
        let (location, _) = get_location_for_offset(self.src, span.start);
        if !self.dead.insert(identifier.clone()) {
            println!(
                "ERROR: Dead identifier '{}' used on line {}.",
                identifier, location.line
            );
        } else {
            if self.debug_prints {
                println!(
                    "Killed identifier '{}' on line {}.",
                    identifier, location.line
                );
            }
        }
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

    fn make_live(&mut self, identifier: String, &span: &span::Span) {
        // Remove the identifier itself.
        self.dead.remove(&identifier);

        // Remove all identifiers that start with the given identifier in a member "." chain.
        let period = identifier.clone() + ".";
        let arrow = identifier.clone() + "->";
        self.dead
            .retain(|x| !x.starts_with(&period) && !x.starts_with(&arrow));

        if self.debug_prints {
            let (location, _) = get_location_for_offset(self.src, span.start);
            println!(
                "Made live identifier '{}' on line {}.",
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

        if let DeclaratorKind::Identifier(identifier) = &init_declarator.declarator.node.kind.node {
            self.make_live(identifier.node.name.clone(), span)
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
            match &boe.rhs.node {
                Expression::Identifier(rhs) => {
                    self.kill(rhs.node.name.clone(), span);
                }
                _ => visit::visit_expression(self, &boe.rhs.node, span),
            }
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
        self.visit_expression(
            &member_expression.expression.node,
            &member_expression.expression.span,
        );
        self.member_count -= 1;

        // This one doesn't announce if dead because the error message will come from the kill function.
        self.kill_next = false;
        self.visit_identifier(
            &member_expression.identifier.node,
            &member_expression.identifier.span,
        );
        self.member_identifier
            .push(member_expression.identifier.node.name.clone());

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

    // =============================================== Control Flow ===============================================
    // This union logic might be better any time moving into a new statement, check the tree.
    fn visit_if_statement(&mut self, if_statement: &'ast IfStatement, span: &'ast span::Span) {
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
        visit::visit_unary_operator_expression(self, uoe, span);
    }

    // Function calls are not killed (foo(x) does not kill foo).
    fn visit_call_expression(
        &mut self,
        call_expression: &'ast CallExpression,
        span: &'ast span::Span,
    ) {
        self.kill_next = false;
        visit::visit_call_expression(self, call_expression, span);
    }

    fn visit_function_definition(
        &mut self,
        function_definition: &'ast FunctionDefinition,
        span: &'ast span::Span,
    ) {
        self.kill_next = false;
        visit::visit_function_definition(self, function_definition, span);
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
        visit::visit_type_specifier(self, type_specifier, span);
    }

    // Don't kill a struct name during its definition (typdef struct struct_name { ... )
    fn visit_struct_type(&mut self, struct_type: &'ast StructType, span: &'ast span::Span) {
        if struct_type.identifier.is_some() {
            self.kill_next = false;
        }
        visit::visit_struct_type(self, struct_type, span);
    }

    // Don't kill struct members being declared.
    fn visit_struct_field(&mut self, struct_field: &'ast StructField, span: &'ast span::Span) {
        self.kill_next = false;
        visit::visit_struct_field(self, struct_field, span);
    }

    // =============================================== Debug ===============================================
    // For printing the dead set each "line".
    fn visit_block_item(&mut self, block_item: &'ast BlockItem, span: &'ast span::Span) {
        self.print_dead_pre(span);
        visit::visit_block_item(self, block_item, span);
        self.print_dead_post();
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
        debug_prints: true,
    };

    if ownership_checker.debug_prints {
        let s = &mut String::new();
        let mut printer = Printer::new(s);
        printer.visit_translation_unit(&parse.unit);
        println!("{s}");
    }

    ownership_checker.visit_translation_unit(&parse.unit);
}

// RUN                         cargo clippy            to view
// git commit -m ""     ->     cargo clippy --fix      to fix
