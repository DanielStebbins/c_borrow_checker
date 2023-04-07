/*
Rules:
    - Assigning to a variable makes it un-dead.
*/

/*
Ranting:
    - I can't know every field of every struct in an arbitrary program, so it must be possible to kill a label the first time I see it, without it ever having been alive.
    - Avoid unwrap() when possible
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

struct Checker<'a> {
    src: &'a str,
    assignments: i32,
    dead: HashSet<String>,
    kill_next: bool,
}

// Split into several impl's.
impl<'ast, 'a> visit::Visit<'ast> for Checker<'a> {
    // Make identifiers valid if they are on the LHS of an assignment or are declared.
    fn visit_init_declarator(
        &mut self,
        init_declarator: &'ast InitDeclarator,
        span: &'ast span::Span,
    ) {
        visit::visit_init_declarator(self, init_declarator, span);
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
            if let Expression::Identifier(lhs) = &boe.lhs.node {
                self.dead.remove(&lhs.node.name);
                // self.kill_next = false;

                let (location, _) = get_location_for_offset(self.src, span.start);
                println!(
                    "Live-d identifier '{}' on line {}.",
                    lhs.node.name, location.line
                );
            }
        } else {
            // Just don't visit LHS of assignments?
            // Visit the LHS, now that it has been marked valid.
            self.visit_expression(&boe.lhs.node, &boe.lhs.span);
        }
    }

    // Kill any identifier that is used in a context not on the accepted list. (references, function definitions, function calls).
    fn visit_identifier(&mut self, identifier: &'ast Identifier, span: &'ast span::Span) {
        if self.kill_next {
            let (location, _) = get_location_for_offset(self.src, span.start);
            if !self.dead.insert(identifier.name.clone()) {
                println!(
                    "Dead identifier '{}' used on line {}.",
                    identifier.name, location.line
                );
            } else {
                println!(
                    "Killed identifier '{}' on line {}.",
                    identifier.name, location.line
                );
                println!("{:?}", self.dead);
            }
        }
        self.kill_next = true;
        visit::visit_identifier(self, identifier, span)
    }

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

    // =============================================== No Kills ===============================================
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

    // For printing the dead set each "line".
    fn visit_block_item(&mut self, block_item: &'ast BlockItem, span: &'ast span::Span) {
        println!("{:?}", self.dead);
        visit::visit_block_item(self, block_item, span)
    }
}

fn main() {
    let file_path = "inputs\\ownership0.c";
    let config = Config::default();
    let result = parse(&config, file_path);

    let parse = result.expect("Parsing Error!\n");

    let mut checker = Checker {
        src: &parse.source,
        assignments: 0,
        dead: HashSet::new(),
        kill_next: true,
    };

    let s = &mut String::new();
    let mut printer = Printer::new(s);
    printer.visit_translation_unit(&parse.unit);
    println!("{s}");

    checker.visit_translation_unit(&parse.unit);
    println!("Dead variables at exit: {:?}", checker.dead);
}

// fn check(lines: Vec<String>) {
//     let mut dead: Vec<HashSet<String>> = Vec::new();
//     // Includes periods for struct fields.
//     let variable_regex = Regex::new(r"^(?:[a-zA-Z_][a-zA-Z0-9_.]*)$").unwrap();

//     for line in lines {
//         // Create the mapping of variables for this line.
//         if let Some(last) = dead.last() {
//             dead.push(last.clone());
//         } else {
//             dead.push(HashSet::new());
//         }
//         let set = dead.last_mut().unwrap();

//         let mut killed: HashSet<String> = HashSet::new();
//         let mut unkilled: HashSet<String> = HashSet::new();

//         let equal_index = line.find('=');
//         if equal_index.is_some() && equal_index != line.find("==") {
//             // Line is an assignment.
//             // Split the line before the '=' into tokens, the last one before the '=' is the variable name.
//             let sides = line.split_once('=').unwrap();

//             // RHS of the Assignment.
//             let mut chars = sides.1.trim().chars();
//             chars.next_back();
//             let rhs = chars.as_str();
//             if variable_regex.is_match(rhs) {
//                 // Right Hand Side is a variable name implies this is assignment is creating an alias.
//                 if let Err(err_message) = starts_with_dead(set, rhs) {
//                     println!("{err_message} on RHS of assignment in line '{line}'");
//                 }
//                 killed.insert(rhs.to_string());
//             }

//             // LHS of the Assignment.
//             let lhs = sides.0.split_whitespace().collect::<Vec<_>>();
//             let variable = lhs.last().unwrap();
//             if lhs.len() == 1 {
//                 // Previously declared variable, might be a field of a dead variable.
//                 if let Err(err_message) = starts_with_dead_lhs(set, variable) {
//                     println!("{err_message} on LHS of assignment in line '{line}'");
//                 }
//                 unkilled.insert(variable.to_string());
//             }
//         } else {
//             // Line is not an assignment.
//             if let Err(err_message) = has_dead(set, line.as_str()) {
//                 println!("{err_message} in line '{line}'");
//             }
//         }

//         // Killing the variables that were passed to functions.
//         // killed.extend(function_arguments_in(&line));

//         // Updating the dead set with this assignment's values.
//         set.extend(killed);
//         set.retain(|variable| !unkilled.contains(variable));
//     }

//     println!("{dead:?}");
// }

// // For the LHS of an assignment, a dead variable should not be flagged. It's fields however, should be.
// fn starts_with_dead_lhs(variables: &HashSet<String>, s: &str) -> Result<(), String> {
//     for variable in variables.iter() {
//         if s.starts_with(variable) {
//             let next_index = variable.len();
//             if next_index < s.len() && s.chars().nth(next_index).unwrap() == '.' {
//                 // Here, we can be sure s contains a reference to a dead variable, but is not a dead variable.
//                 return Err(format!("Found dead variable '{variable}'"));
//             }
//         }
//     }
//     Ok(())
// }

// fn starts_with_dead(variables: &HashSet<String>, s: &str) -> Result<(), String> {
//     for variable in variables.iter() {
//         if s.starts_with(variable) {
//             let next_index = variable.len();
//             if next_index >= s.len() || s.chars().nth(next_index).unwrap() == '.' {
//                 // Here, we can be sure s contains a reference to a dead variable.
//                 return Err(format!("Found dead variable '{variable}'"));
//             }
//         }
//     }
//     Ok(())
// }

// // If needs more types of errors, make an enum for it.
// fn has_dead(variables: &HashSet<String>, s: &str) -> Result<(), String> {
//     // Checks if character before is a piece of a variable name or a period (not dead if match).
//     let before = Regex::new(r"[a-zA-Z0-9_.]+").unwrap();

//     // Checks if character after is a piece of a variable (not dead if match).
//     let after = Regex::new(r"[a-zA-Z0-9_]+").unwrap();

//     for variable in variables.iter() {
//         // If s contains the current variable.
//         if let Some(index) = s.find(variable) {
//             let next_index = index + variable.len();
//             if (index < 1 || !before.is_match(&s[index - 1..index]))
//                 && (next_index >= s.len() || !after.is_match(&s[next_index..next_index + 1]))
//             {
//                 // Here, we can be sure s contains a reference to a dead variable based on the regex checks.
//                 return Err(format!("Found dead variable '{variable}'"));
//             }
//         }
//     }
//     Ok(())
// }

// fn function_arguments_in(line: &str) -> HashSet<String> {
//     let function_char_regex = Regex::new(r"[a-zA-Z0-9_]$").unwrap();

//     let killed: HashSet<String> = HashSet::new();
//     let split = line.split(['(', ')'].as_ref());
//     for element in split {
//         if function_char_regex.is_match(element) {
//             // Last character is part of an identifier.
//         }
//     }
//     killed
// }

// RUN                         cargo clippy            to view
// git commit -m ""     ->     cargo clippy --fix      to fix
