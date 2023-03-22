/*
Source (C) Code Assumptions:
    - Valid C syntax (compiles).
    - No multi-line comments.
    - { and } on their own lines.
    - No variable name shadowing (TODO).
    - Assignments have a single = as the first = on their line.
    - One statement per line (0 or 1 semicolons per line).
*/

/*
Ranting:
    - I can't know every field of every struct in an arbitrary program, so it must be possible to kill a label the first time I see it, without it ever having been alive.
*/

use std::collections::HashSet;
use std::fs;

fn main() {
    let file_path = "..\\C_Programs\\ownership_smallest.c";
    let lines = read_file(file_path);

    // let mut variables: Vec<HashMap<String, String>> = Vec::new();
    // let mut dead: Vec<String> = Vec::new();
    // let mut variables: Vec<HashMap<String, bool>> = Vec::new();
    let mut dead: Vec<HashSet<String>> = Vec::new();
    for line in lines {
        // Create the mapping of variables for this line.
        if dead.len() == 0 {
            dead.push(HashSet::new());
        } else {
            dead.push(dead.last().unwrap().clone());
        }
        let set = dead.last_mut().unwrap();

        // Look for new variable assignments.
        let equal_index = line.find('=');
        if equal_index != None && equal_index != line.find("==") {
            // Split the line before the '=' into tokens, the last one before the '=' is the variable name.
            let split = line.split_whitespace().collect::<Vec<_>>();

            let list_eq_index = split.iter().position(|&r| r == "=").unwrap();
            println!("{:?}", split);

            // let variable = split[list_eq_index - 1].to_string();

            // If there is one token after the equals (likely aliasing).
            if list_eq_index + 2 == split.len() {
                // Remove ';' from the end.
                let mut chars = split.last().unwrap().chars();
                chars.next_back();
                let right_token = chars.collect::<String>();
                println!("{:?}", right_token);
                set.insert(right_token);
            }

            // Add a mapping in the variables HashMap.
            // map.insert(variable, true);
        }
    }

    println!("{:?}", dead)
}

fn read_file(path: &str) -> Vec<String> {
    let contents = fs::read_to_string(path).expect("Could not read source file");

    let raw_lines = contents.split("\r\n");
    let mut lines: Vec<String> = Vec::new();
    for line in raw_lines {
        let trimmed_line = line.trim();

        if !trimmed_line.is_empty()
            && (trimmed_line.len() < 2 || trimmed_line[..2] != "//".to_string())
        {
            lines.push(line.trim().to_string());
        }
    }
    return lines;
}
