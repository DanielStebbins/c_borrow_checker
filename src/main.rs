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
    - Support += (AssignPlus) and other shorthand.
    - Count . and -> separately.
    - Clean the messy global scope.
    - Reference errors.
    - Lifetime errors.
    - Cannot move out of index of array.
*/

#![feature(iter_intersperse)]

// use crate::borrow_checker::BorrowChecker;

mod ast_traversal;
mod borrow_checker;
mod variable;

use borrow_checker::BorrowChecker;
use borrow_checker::PrintType;

use lang_c::driver::*;
use lang_c::print::*;
use lang_c::visit::*;

use std::collections::HashMap;

fn main() {
    let file_path = "inputs\\kernel0.c";
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

        next_ref_const: false,

        set_prints: PrintType::Ownership,
        event_prints: PrintType::ErrorOnly,
    };

    // let s = &mut String::new();
    // let mut printer = Printer::new(s);
    // printer.visit_translation_unit(&parse.unit);
    // println!("{s}");

    ownership_checker.visit_translation_unit(&parse.unit);
}

// RUN                         cargo clippy            to view
// git commit -m ""     ->     cargo clippy --fix      to fix
