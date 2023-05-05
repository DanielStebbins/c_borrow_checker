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
    - C2Rust converter not very helpful because it uses 'unsafe' to avoid normal rust checks.
*/

/*
Limitations:
    - void pointers assume pointing to Copy types, so they become &i32 when converting to Rust.

    - Rust places extra restrictions on globals, so I passed them in as function parameters instead.
    - Some unused struct fields that would require additional copy-pasting have been omitted. These have no effect on the output.
    - Parser cannot parse <stdlib.h>, so tests with malloc and free are not possible.
*/

#![feature(iter_intersperse)]

mod ast_traversal;
mod borrow_checker;
mod variable;

use borrow_checker::BorrowChecker;
use borrow_checker::PrintType;

use lang_c::driver::*;
use lang_c::print::*;
use lang_c::visit::*;

use std::io::Write;

fn main() {
    let file_path = "inputs\\kernel0\\round0.c";
    let config = Config::default();
    let result = parse(&config, file_path);

    let parse = result.expect("Parsing Error!\n");

    let mut borrow_checker = BorrowChecker::new(
        vec!["perf_event_max_stack_handler".to_string()],
        &parse.source,
        false,
        PrintType::ErrorOnly,
        PrintType::ErrorOnly,
    );

    // Running the checker.
    borrow_checker.visit_translation_unit(&parse.unit);
    println!("\n\n"); // Spacing to make it easier to get images of the output.

    // Printing the abstract syntax tree to a file.
    let s = &mut String::new();
    let mut printer = Printer::new(s);
    printer.visit_translation_unit(&parse.unit);
    let mut file = std::fs::File::create("ast.txt").expect("AST file creation failed");
    file.write_all(s.as_bytes()).expect("AST file write failed");
}

// RUN                         cargo clippy            to view
// git commit -m ""     ->     cargo clippy --fix      to fix
