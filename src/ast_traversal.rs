use crate::variable::*;
use crate::BorrowChecker;
use crate::PrintType;
use lang_c::ast::*;
use lang_c::*;
use std::collections::HashMap;

impl<'ast, 'a> visit::Visit<'ast> for BorrowChecker<'a> {
    // Scope changes.
    fn visit_statement(&mut self, statement: &'ast Statement, span: &'ast span::Span) {
        // Add a new scope layer for this block.
        if !self.function_body {
            if let Statement::Compound(_) = statement {
                self.scopes.push(HashMap::new());
            }
        }
        self.function_body = false;

        // Run the block.
        let before_scope = self.scopes.clone();
        visit::visit_statement(self, statement, span);

        // Merge the scopes, then remove the block's scope layer.
        if let Statement::Compound(_) = statement {
            self.merge_scopes(&before_scope);
            self.scopes.pop();
        }
    }

    // Variable declarations.
    fn visit_declaration(&mut self, declaration: &'ast Declaration, _: &'ast span::Span) {
        for declarator in &declaration.declarators {
            self.declare_variable(
                &declarator.node.declarator.node,
                &declaration.specifiers,
                false,
            )
        }
        for declarator in &declaration.declarators {
            self.visit_init_declarator(&declarator.node, &declarator.span);
        }
    }

    // Assignments that happen directly after a declaration.
    fn visit_init_declarator(
        &mut self,
        init_declarator: &'ast InitDeclarator,
        span: &'ast span::Span,
    ) {
        // Declaring the LHS variable is done in visit_declaration.

        // RHS
        if let Some(ref initializer) = init_declarator.initializer {
            match &initializer.node {
                Initializer::Expression(expression) => {
                    self.set_expression_ownership(&expression, false, span);
                }
                _ => visit::visit_initializer(self, &initializer.node, span),
            }
        }

        // The LHS is potentially a new reference to the RHS.
        if let DeclaratorKind::Identifier(identifier) = &init_declarator.declarator.node.kind.node {
            if let Some(ref initializer) = init_declarator.initializer {
                if let Initializer::Expression(expression) = &initializer.node {
                    self.add_reference(identifier.node.name.clone(), &expression, span);
                }
            }
        }
    }

    // Assignments
    fn visit_binary_operator_expression(
        &mut self,
        boe: &'ast ast::BinaryOperatorExpression,
        span: &'ast span::Span,
    ) {
        if boe.operator.node != BinaryOperator::Assign {
            visit::visit_binary_operator_expression(self, boe, span);
        } else {
            self.set_expression_ownership(&boe.rhs, false, span);
            self.set_expression_ownership(&boe.lhs, true, span);
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

    // For things declared at the global scope (function prototypes, struct definitions, global variables).
    fn visit_external_declaration(
        &mut self,
        external_declaration: &'ast ExternalDeclaration,
        _: &'ast span::Span,
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
                let mut no_visit = false;
                // For struct definitions, which we use to know the types of undeclared struct members.
                for specifier in &declaration.node.specifiers {
                    if let DeclarationSpecifier::TypeSpecifier(type_specifier) = &specifier.node {
                        if let TypeSpecifier::Struct(struct_type) = &type_specifier.node {
                            if let Some(struct_identifier) = &struct_type.node.identifier {
                                let struct_name = &struct_identifier.node.name;
                                if !self.structs.contains_key(struct_name) {
                                    self.add_struct(declaration);
                                    return;
                                }
                            }
                        }
                    }
                    if let DeclarationSpecifier::StorageClass(storage_class) = &specifier.node {
                        if let StorageClassSpecifier::Typedef = &storage_class.node {
                            // To stop typedefs from being treated as variables.
                            no_visit = true;
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
                            return;
                        }
                    }
                }
                // Creates global variables.
                if !no_visit {
                    self.visit_declaration(&declaration.node, &declaration.span);
                }
            }
            _ => {}
        }
    }

    // Function Definitions.
    fn visit_function_definition(
        &mut self,
        function_definition: &'ast FunctionDefinition,
        _: &'ast span::Span,
    ) {
        if let DeclaratorKind::Identifier(id) = &function_definition.declarator.node.kind.node {
            // Ignore any function definitions that the user did not specify to be checked.
            if self.functions_to_check.contains(&id.node.name) {
                // Functions add the new scope early so it can include all their parameters.
                self.function_body = true;
                self.scopes.push(HashMap::new());

                // Copied from visit::visit_function_definition to replace the declarator visit with only visiting the derived declarators (the function name is not a variable).
                for derived_declarator in &function_definition.declarator.node.derived {
                    self.visit_derived_declarator(
                        &derived_declarator.node,
                        &derived_declarator.span,
                    )
                }
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

    // Parameters passed to a function are declared in the function's scope.
    fn visit_parameter_declaration(
        &mut self,
        parameter_declaration: &'ast ParameterDeclaration,
        _: &'ast span::Span,
    ) {
        if let Some(declarator) = &parameter_declaration.declarator {
            self.declare_variable(&declarator.node, &parameter_declaration.specifiers, true);
        }
    }

    // Function calls, like foo(x);
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

        // Decide which action to take on each of the function's arguments.
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
                    } else {
                        // useful for *p (dereferencing).
                        self.visit_unary_operator_expression(&uo.node, &uo.span);
                    }
                }
                _ => {
                    // If not a reference, try to set as not owner. Won't do anything if it isn't an Owner type.
                    self.visit_expression(&argument.node, &argument.span);
                    self.set_expression_ownership(argument, false, span);
                }
            }
            argument_index += 1;
        }
    }

    // Reset the previous member identifier so other functions know a new expression has started.
    fn visit_expression(&mut self, expression: &'ast Expression, span: &'ast span::Span) {
        self.member_identifier.clear();
        visit::visit_expression(self, expression, span);
    }

    // Every identifier, like x
    fn visit_identifier(&mut self, identifier: &'ast Identifier, span: &'ast span::Span) {
        if self.member_count > 0 {
            // A struct member is currently being compiled.
            self.member_identifier_pieces.push(identifier.name.clone());
            self.announce_no_ownership(self.member_identifier_pieces.join("."), span);
            self.announce_invalid_reference(self.member_identifier_pieces.join("."), span);
        } else {
            // Non-struct member identifier.
            self.announce_no_ownership(identifier.name.clone(), span);
            self.announce_invalid_reference(identifier.name.clone(), span);
        }
    }

    // Recursively visits all the nodes that make up a struct member identifer.
    fn visit_member_expression(
        &mut self,
        member_expression: &'ast MemberExpression,
        span: &'ast span::Span,
    ) {
        // Compiling a member identifier (struct_name.x.y ..., if struct_name.x lacks ownership, it's an error).
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
            // These run every time, except possibly the last step if muted.
            let partial_name = self.member_identifier_pieces.join(".");
            self.announce_no_ownership(partial_name.clone(), span);
            self.announce_invalid_reference(partial_name, span);
        }
        if self.member_count == 0 {
            // This is done once after the entire recurisve call chain.
            self.member_identifier = self.member_identifier_pieces.join(".");
            self.member_identifier_pieces.clear();
        }
    }

    // For indirection (dereferencing). Address operator & handled elsewhere.
    fn visit_unary_operator_expression(
        &mut self,
        uoe: &'ast UnaryOperatorExpression,
        span: &'ast span::Span,
    ) {
        match &uoe.operator.node {
            UnaryOperator::Indirection => {
                self.dereference_name.clear();
                match &uoe.operand.node {
                    Expression::Identifier(id) => {
                        self.announce_invalid_reference(id.node.name.to_string(), span);
                        let var = self.name_to_var(&id.node.name);
                        match &var.var_type {
                            VarType::ConstRef(points_to) | VarType::MutRef(points_to) => {
                                let pointed_to = points_to.iter().next().unwrap();
                                self.dereference_name = pointed_to.name.clone();
                            }
                            _ => {}
                        }
                    }
                    Expression::Member(member_expression) => {
                        let member_pieces_backup = self.member_identifier_pieces.clone();
                        let member_count_backup = self.member_count;
                        self.get_member_expression_identifier(member_expression);
                        self.announce_invalid_reference(self.member_identifier.clone(), span);
                        let var = self.name_to_var(&self.member_identifier.clone());
                        match &var.var_type {
                            VarType::ConstRef(points_to) | VarType::MutRef(points_to) => {
                                let pointed_to = points_to.iter().next().unwrap();
                                self.dereference_name = pointed_to.name.clone();
                            }
                            _ => {}
                        }
                        self.member_identifier_pieces = member_pieces_backup;
                        self.member_count = member_count_backup;
                    }
                    _ => self.visit_expression(&uoe.operand.node, &uoe.operand.span),
                }
            }
            _ => visit::visit_unary_operator_expression(self, uoe, span),
        }
        // If this dereference is part of a member expression, add the result to the member expression name.
        if self.member_count > 0 {
            self.member_identifier_pieces
                .push(self.dereference_name.clone());
        }
    }

    // If statements - different from loop scopes because the if and else blocks have to be treated equally.
    fn visit_if_statement(&mut self, if_statement: &'ast IfStatement, _: &'ast span::Span) {
        self.visit_expression(&if_statement.condition.node, &if_statement.condition.span);

        let temp = self.scopes.clone();
        self.visit_statement(
            &if_statement.then_statement.node,
            &if_statement.then_statement.span,
        );
        if let Some(ref else_statement) = if_statement.else_statement {
            let then_scopes = self.scopes.clone();

            // Runs the else block as if the if block has not yet been run.
            self.scopes = temp;
            self.visit_statement(&else_statement.node, &else_statement.span);
            self.merge_scopes(&then_scopes);
        }
    }

    // For printing requested outputs on each line. Requires each line be a new block item, which requires {} around every block.
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
