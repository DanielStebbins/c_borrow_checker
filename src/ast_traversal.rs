use crate::variable::*;
use crate::BorrowChecker;
use crate::PrintType;
use lang_c::ast::*;
use lang_c::*;
use std::collections::HashMap;
use std::collections::HashSet;

impl<'ast, 'a> visit::Visit<'ast> for BorrowChecker<'a> {
    // Triggers scope changes.
    fn visit_statement(&mut self, statement: &'ast Statement, span: &'ast span::Span) {
        // Add a new scope layer for this block.
        if !self.function_body {
            if let Statement::Compound(_) = statement {
                self.scopes.push(HashMap::new());
            }
        }
        self.function_body = false;

        // Run the block.
        visit::visit_statement(self, statement, span);

        // Remove the block's scope layer.
        if let Statement::Compound(_) = statement {
            let scope = self.scopes.len() - 1;
            let lost: Vec<String> = self
                .scopes
                .last()
                .unwrap()
                .keys()
                .map(|k| k.to_string())
                .collect();

            // Control flow rules applied at the end of block to enfore strictness.
            for name in lost {
                let id = Id {
                    name: name,
                    scope: scope,
                };
                let const_ids = self.id_to_var(&id).const_refs.clone();
                for ref_id in const_ids.iter() {
                    if let VarType::ConstRef(points_to) = &mut self.id_to_mut_var(ref_id).var_type {
                        points_to.remove(&id);
                    }
                }
                let mut_ids = self.id_to_var(&id).mut_refs.clone();
                for ref_id in mut_ids.iter() {
                    if let VarType::MutRef(points_to) = &mut self.id_to_mut_var(ref_id).var_type {
                        points_to.remove(&id);
                    }
                }
            }
            self.scopes.pop();
        }
    }

    // =============================================== Assignments (make_live) ===============================================
    fn visit_declaration(&mut self, declaration: &'ast Declaration, span: &'ast span::Span) {
        for declarator in &declaration.declarators {
            self.declare_variable(
                &declarator.node.declarator.node,
                &declaration.specifiers,
                false,
            )
        }
        // Stolen from visit::visit_declaration, edited to stop visiting specifiers (it was trying to create variables for type identifiers.)
        for declarator in &declaration.declarators {
            self.visit_init_declarator(&declarator.node, &declarator.span);
        }
    }

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
                    self.set_expression_is_valid(&expression, false, span);
                }
                _ => visit::visit_initializer(self, &initializer.node, span),
            }
        }

        // LHS
        if let DeclaratorKind::Identifier(identifier) = &init_declarator.declarator.node.kind.node {
            // Possibly adding a reference to the RHS, which requires the LHS's identifier.
            if let Some(ref initializer) = init_declarator.initializer {
                if let Initializer::Expression(expression) = &initializer.node {
                    self.add_reference(identifier.node.name.clone(), &expression, span);
                }
            }
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
            self.set_expression_is_valid(&boe.rhs, false, span);
            self.set_expression_is_valid(&boe.lhs, true, span);
            match &boe.lhs.node {
                Expression::Identifier(name) => {
                    self.add_reference(name.node.name.clone(), &boe.rhs, span);
                }
                Expression::Member(_) => {
                    // member identifier is known from when it was set to valid in set_expression_is_valid.
                    self.add_reference(self.member_identifier.clone(), &boe.rhs, span);
                }
                _ => {}
            }
        }
    }

    // For things declared at the global scope. Currently only visits function defintions.
    // Should visit other globals but w/o applying borrow checker rules to them.
    fn visit_external_declaration(
        &mut self,
        external_declaration: &'ast ExternalDeclaration,
        span: &'ast span::Span,
    ) {
        match external_declaration {
            ExternalDeclaration::FunctionDefinition(function_definition) => {
                // For function definitions, which we might want to check.
                self.visit_function_definition(
                    &function_definition.node,
                    &function_definition.span,
                );
            }
            ExternalDeclaration::Declaration(declaration) => {
                // For struct definitions, which we use to know the types of undeclared struct members.
                for specifier in &declaration.node.specifiers {
                    if let DeclarationSpecifier::TypeSpecifier(type_specifier) = &specifier.node {
                        if let TypeSpecifier::Struct(_) = &type_specifier.node {
                            self.add_struct(declaration);
                        }
                    }
                }

                // For function declarations, which we use to know what to do at each function call.
                for init_declarator in &declaration.node.declarators {
                    for derived_declarator in &init_declarator.node.declarator.node.derived {
                        if let DerivedDeclarator::Function(function_declarator) =
                            &derived_declarator.node
                        {
                            self.add_function(
                                &init_declarator.node.declarator,
                                &function_declarator.node.parameters,
                            );
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // Ignore any function definitions that the user did not specify to be checked.
    fn visit_function_definition(
        &mut self,
        function_definition: &'ast FunctionDefinition,
        span: &'ast span::Span,
    ) {
        if let DeclaratorKind::Identifier(id) = &function_definition.declarator.node.kind.node {
            if self.functions_to_check.contains(&id.node.name) {
                // Functions add the new scope early so it can include all their parameters.
                self.function_body = true;
                self.scopes.push(HashMap::new());

                // Copied from visit::visti_function_definition to remove the declarator visit (the function name is not a variable).
                for specifier in &function_definition.specifiers {
                    self.visit_declaration_specifier(&specifier.node, &specifier.span);
                }
                for declaration in &function_definition.declarations {
                    self.visit_declaration(&declaration.node, &declaration.span);
                }
                self.visit_statement(
                    &function_definition.statement.node,
                    &function_definition.statement.span,
                );
            }
        }
    }

    // // Parameters passed to a function are made live.
    fn visit_parameter_declaration(
        &mut self,
        parameter_declaration: &'ast ParameterDeclaration,
        _: &'ast span::Span,
    ) {
        if let Some(declarator) = &parameter_declaration.declarator {
            self.declare_variable(&declarator.node, &parameter_declaration.specifiers, true);
        }
    }

    // Function calls in checked funtions, like foo(x);
    fn visit_call_expression(
        &mut self,
        call_expression: &'ast CallExpression,
        span: &'ast span::Span,
    ) {
        // Does not visit the function name expression, to avoid creating a variable for the function identifier.
        let Expression::Identifier(function_id) = &call_expression.callee.node else {
            return;
        };

        let function_name = &function_id.node.name;
        let Some(function_parameters) = self.functions.get(function_name) else {
            println!("ISSUE: Function name '{function_name}' not defined!");
            return;
        };
        let parameters_clone = function_parameters.clone();
        let mut argument_index = 0;
        for argument in &call_expression.arguments {
            match &argument.node {
                Expression::UnaryOperator(uo) => {
                    if UnaryOperator::Address == uo.node.operator.node {
                        if let Expression::Identifier(identifier) = &uo.node.operand.node {
                            // The argument looks like &x.
                            if argument_index > parameters_clone.len()
                                || matches!(parameters_clone[argument_index], VarType::MutRef(_))
                            {
                                // Passing a mutable reference makes all previous mut and const references invalid.
                                let var = self.name_to_mut_var(&identifier.node.name);
                                var.const_refs.clear();
                                var.mut_refs.clear();
                            } else {
                                // Passing a const reference makes all previous mut references invalid.
                                let var = self.name_to_mut_var(&identifier.node.name);
                                var.mut_refs.clear();
                            }
                        }
                    }
                }
                _ => {
                    // If not a reference, try to set as not owner. Won't do anything if it isn't an owner type.
                    self.set_expression_is_valid(argument, false, span);
                } // parameters_clone[argument_index]
                  // VarType::Copy => {}
                  // VarType::Owner(_, _) => {
                  //     self.set_expression_is_valid(argument, false, span);
                  // }
                  // VarType::ConstRef(_) => {
                  //     if let Expression::UnaryOperator(uo) = &argument.node {
                  //         if UnaryOperator::Address == uo.node.operator.node {
                  //             // The argument looks like &x.
                  //             if let Expression::Identifier(identifier) = &uo.node.operand.node {
                  //                 // Passing a const reference makes all previous mut references invalid.
                  //                 let var = self.name_to_mut_var(&identifier.node.name);
                  //                 var.mut_refs.clear();
                  //             }
                  //         }
                  //     }
                  // }
                  // VarType::MutRef(_) => {
                  //     if let Expression::UnaryOperator(uo) = &argument.node {
                  //         if UnaryOperator::Address == uo.node.operator.node {
                  //             // The argument looks like &x.
                  //             if let Expression::Identifier(identifier) = &uo.node.operand.node {
                  //                 // Passing a mutable reference makes all previous mut and const references invalid.
                  //                 let var = self.name_to_mut_var(&identifier.node.name);
                  //                 var.const_refs.clear();
                  //                 var.mut_refs.clear();
                  //             }
                  //         }
                  //     }
                  // }
            }
            argument_index += 1;
        }
    }

    // // =============================================== Small Expressions ===============================================

    fn visit_identifier(&mut self, identifier: &'ast Identifier, span: &'ast span::Span) {
        if self.member_count > 0 {
            self.member_identifier_pieces.push(identifier.name.clone());
            self.announce_no_ownership(self.member_identifier_pieces.join("."), span);
            self.announce_invalid_reference(self.member_identifier_pieces.join("."), span);
        } else {
            self.announce_no_ownership(identifier.name.clone(), span);
            self.announce_invalid_reference(identifier.name.clone(), span);
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
            self.announce_no_ownership(self.member_identifier_pieces.join("."), span);
        }
        if self.member_count == 0 {
            self.member_identifier = self.member_identifier_pieces.join(".");
            self.member_identifier_pieces.clear();
        }
    }

    // // =============================================== References ===============================================

    // // =============================================== Control Flow ===============================================
    // This union logic might be better any time moving into a new statement, check the tree.
    fn visit_if_statement(&mut self, if_statement: &'ast IfStatement, _: &'ast span::Span) {
        self.visit_expression(&if_statement.condition.node, &if_statement.condition.span);

        let temp = self.scopes.clone();
        self.visit_statement(
            &if_statement.then_statement.node,
            &if_statement.then_statement.span,
        );
        if let Some(ref else_statement) = if_statement.else_statement {
            let then_scopes = self.scopes.clone();
            self.scopes = temp;
            self.visit_statement(&else_statement.node, &else_statement.span);
            self.merge_scopes(&then_scopes);
        }
    }

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
