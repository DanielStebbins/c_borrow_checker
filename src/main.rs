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
    - Return error.
    - Cannot move out of index of array.
    - Struct members that haven't been seen before are assigned a type based on their parent struct's entry in the struct mapping.
    - reference assignment errors / not errors
    - &struct.member borrows whole struct.
    - Ownership of dereferenced pointer values?
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
    let file_path = "inputs\\ownership1.c";
    let config = Config::default();
    let result = parse(&config, file_path);

    let parse = result.expect("Parsing Error!\n");

    let mut ownership_checker =
        BorrowChecker::new(&parse.source, PrintType::ErrorOnly, PrintType::ErrorOnly);

    // Running the checker.
    ownership_checker.visit_translation_unit(&parse.unit);

    // Printing the abstract syntax tree to a file.
    let s = &mut String::new();
    let mut printer = Printer::new(s);
    printer.visit_translation_unit(&parse.unit);
    let mut file = std::fs::File::create("ast.txt").expect("AST file creation failed");
    file.write_all(s.as_bytes()).expect("AST file write failed");
}

// RUN                         cargo clippy            to view
// git commit -m ""     ->     cargo clippy --fix      to fix
