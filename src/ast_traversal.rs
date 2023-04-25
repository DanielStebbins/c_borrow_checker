use crate::variable::VarType;
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
                            )
                        } else {
                            self.declare_variable(
                                identifier.node.name.clone(),
                                VarType::MutRef(HashSet::new()),
                            )
                        }
                    }
                    _ => {}
                }
            }

            // Possibly adding a reference, which requires the LHS's identifier.
            // if let Some(ref initializer) = init_declarator.initializer {
            //     if let Initializer::Expression(expression) = &initializer.node {
            //         self.add_if_reference(identifier.node.name.clone(), &expression, span);
            //     }
            // }
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
            self.set_expression_is_valid(argument, false, span);
        }
    }

    // // =============================================== Small Expressions ===============================================

    fn visit_identifier(&mut self, identifier: &'ast Identifier, span: &'ast span::Span) {
        if self.member_count > 0 {
            self.member_identifier_pieces.push(identifier.name.clone());
            self.announce_if_dead(self.member_identifier_pieces.join("."), span);
        } else {
            self.announce_if_dead(identifier.name.clone(), span);
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
            self.announce_if_dead(self.member_identifier_pieces.join("."), span);
        }
        if self.member_count == 0 {
            self.member_identifier = self.member_identifier_pieces.join(".");
            self.member_identifier_pieces.clear();
        }
    }

    // // =============================================== References ===============================================

    // // References are not killed (&x does not kill x).
    // fn visit_unary_operator_expression(
    //     &mut self,
    //     uoe: &'ast UnaryOperatorExpression,
    //     span: &'ast span::Span,
    // ) {
    //     if let Expression::Identifier(operand) = &uoe.operand.node {
    //         if uoe.operator.node == UnaryOperator::Address {
    //             self.announce_if_new_ref_and_mutable(&operand.node.name, span)
    //         }
    //     }
    //     visit::visit_unary_operator_expression(self, uoe, span);
    // }

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
