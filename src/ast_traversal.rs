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
        if let Statement::Compound(_) = statement {
            self.scopes.push(HashMap::new());
        }

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
            // Matches with the first derived declarator, there are more.
            if init_declarator.declarator.node.derived.is_empty() {
                self.declare_variable(identifier.node.name.clone(), VarType::Owner(true))
            } else {
                match &init_declarator.declarator.node.derived[0].node {
                    DerivedDeclarator::Pointer(_) => {
                        if self.next_ref_const {
                            self.declare_variable(
                                identifier.node.name.clone(),
                                VarType::ConstRef(HashSet::new()),
                            );
                        } else {
                            self.declare_variable(
                                identifier.node.name.clone(),
                                VarType::MutRef(HashSet::new()),
                            )
                        }

                        // Possibly adding a reference to the RHS, which requires the LHS's identifier.
                        if let Some(ref initializer) = init_declarator.initializer {
                            if let Initializer::Expression(expression) = &initializer.node {
                                self.add_reference(identifier.node.name.clone(), &expression, span);
                            }
                        }
                    }
                    _ => {}
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

    // // Parameters passed to a function are made live.
    fn visit_parameter_declaration(
        &mut self,
        parameter_declaration: &'ast ParameterDeclaration,
        _: &'ast span::Span,
    ) {
        if let Some(declarator) = &parameter_declaration.declarator {
            if let DeclaratorKind::Identifier(identifier) = &declarator.node.kind.node {
                // self.declare_variable(identifier.node.name.clone());
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
            if let Expression::UnaryOperator(uo) = &argument.node {
                if UnaryOperator::Address == uo.node.operator.node {
                    if let Expression::Identifier(identifier) = &uo.node.operand.node {
                        let var = self.name_to_mut_var(&identifier.node.name);
                        var.const_refs.clear();
                        var.mut_refs.clear();
                    }
                }
            }
            self.set_expression_is_valid(argument, false, span);
        }
    }

    // When 'const' is seen, make it so the next reference is const.
    fn visit_type_qualifier(&mut self, type_qualifier: &'ast TypeQualifier, _: &'ast span::Span) {
        if matches!(type_qualifier, TypeQualifier::Const) {
            self.next_ref_const = true;
        }
    }

    // Reset the flag so after this declaration, pointers are no longer marked const.
    fn visit_declaration(&mut self, declaration: &'ast Declaration, span: &'ast span::Span) {
        visit::visit_declaration(self, declaration, span);
        self.next_ref_const = false;
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
