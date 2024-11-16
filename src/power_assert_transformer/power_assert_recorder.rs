use swc_core::{common::DUMMY_SP, ecma::ast::*};

/// Returns the AST for the `class _powerAssertRecorder` which is injected into files that use power-assert
pub fn power_assert_recorder_definition() -> Stmt {
    macro_rules! this_expr {
        () => {
            Box::new(Expr::This(ThisExpr { span: DUMMY_SP }))
        };
    }
    macro_rules! ident_name {
        ($sym: literal) => {
            IdentName {
                span: DUMMY_SP,
                sym: $sym.into(),
            }
            .into()
        };
    }
    macro_rules! from_ident {
        ($sym: literal) => {
            Into::<Ident>::into($sym).into()
        };
    }
    // class _powerAssertRecorder
    let class = ClassDecl {
        ident: "_powerAssertRecorder".into(),
        class: Box::new(Class {
            body: vec![
                // captured = [];
                ClassMember::ClassProp(ClassProp {
                    key: from_ident!("captured"),
                    value: Some(Box::new(Expr::Array(Default::default()))),
                    ..Default::default()
                }),
                // _capt method
                ClassMember::Method(ClassMethod {
                    key: from_ident!("_capt"),
                    function: Box::new(Function {
                        params: vec![from_ident!("value"), from_ident!("espath")],
                        body: Some(BlockStmt {
                            stmts: vec![
                                // this.captured.push({...})
                                Stmt::Expr(ExprStmt {
                                    expr: Box::new(Expr::Call(CallExpr {
                                        callee: Callee::Expr(Box::new(Expr::Member(MemberExpr {
                                            obj: Box::new(Expr::Member(MemberExpr {
                                                obj: this_expr!(),
                                                prop: ident_name!("captured"),
                                                ..Default::default()
                                            })),
                                            prop: ident_name!("push"),
                                            ..Default::default()
                                        }))),
                                        args: vec![ExprOrSpread {
                                            spread: None,
                                            expr: Box::new(Expr::Object(ObjectLit {
                                                span: DUMMY_SP,
                                                props: vec![
                                                    PropOrSpread::Prop(Box::new(from_ident!(
                                                        "value"
                                                    ))),
                                                    PropOrSpread::Prop(Box::new(from_ident!(
                                                        "espath"
                                                    ))),
                                                ],
                                            })),
                                        }],
                                        ..Default::default()
                                    })),
                                    ..Default::default()
                                }),
                                // return value
                                Stmt::Return(ReturnStmt {
                                    arg: Some(Box::new(ident_name!("value"))),
                                    ..Default::default()
                                }),
                            ],
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                // _expr method
                ClassMember::Method(ClassMethod {
                    key: ident_name!("_expr"),
                    function: Box::new(Function {
                        params: vec![ident_name!("value"), ident_name!("source")],
                        body: Some(BlockStmt {
                            stmts: vec![
                                // const capturedValues = this.captured;
                                Stmt::Decl(Decl::Var(Box::new(VarDecl {
                                    kind: VarDeclKind::Const,
                                    decls: vec![VarDeclarator {
                                        span: DUMMY_SP,
                                        name: ident_name!("capturedValues"),
                                        init: Some(Box::new(Expr::Member(MemberExpr {
                                            span: DUMMY_SP,
                                            obj: this_expr!(),
                                            prop: ident_name!("captured"),
                                        }))),
                                        definite: false,
                                    }],
                                    ..Default::default()
                                }))),
                                // this.captured = [];
                                Stmt::Expr(ExprStmt {
                                    span: DUMMY_SP,
                                    expr: Box::new(Expr::Assign(AssignExpr {
                                        span: DUMMY_SP,
                                        op: AssignOp::Assign,
                                        left: AssignTarget::Simple(SimpleAssignTarget::Member(
                                            MemberExpr {
                                                span: DUMMY_SP,
                                                obj: this_expr!(),
                                                prop: ident_name!("captured"),
                                            },
                                        )),
                                        right: Box::new(Expr::Array(ArrayLit {
                                            span: DUMMY_SP,
                                            elems: vec![],
                                        })),
                                    })),
                                }),
                                // return {...}
                                Stmt::Return(ReturnStmt {
                                    span: DUMMY_SP,
                                    arg: Some(Box::new(Expr::Object(ObjectLit {
                                        span: DUMMY_SP,
                                        props: vec![
                                            PropOrSpread::Prop(Box::new(Prop::KeyValue(
                                                KeyValueProp {
                                                    key: ident_name!("powerAssertContext"),
                                                    value: Box::new(Expr::Object(ObjectLit {
                                                        span: DUMMY_SP,
                                                        props: vec![
                                                            Prop::Shorthand(ident_name!("value"))
                                                                .into(),
                                                            Prop::KeyValue(KeyValueProp {
                                                                key: ident_name!("events"),
                                                                value: Box::new(ident_name!(
                                                                    "capturedValues"
                                                                )),
                                                            })
                                                            .into(),
                                                        ],
                                                    })),
                                                },
                                            ))),
                                            PropOrSpread::Prop(Box::new(from_ident!("source"))),
                                        ],
                                    }))),
                                }),
                            ],
                            ..Default::default()
                        }),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
            ],
            ..Default::default()
        }),
        declare: false,
    };
    Stmt::Decl(Decl::Class(class))
}

#[cfg(test)]
mod test {
    use super::*;
    use swc_core::ecma::{
        transforms::testing::test_inline,
        visit::{as_folder, VisitMut},
    };

    struct TestVisitor {}

    impl VisitMut for TestVisitor {
        fn visit_mut_module(&mut self, node: &mut swc_core::ecma::ast::Module) {
            node.body.push(swc_core::ecma::ast::ModuleItem::Stmt(
                power_assert_recorder_definition(),
            ));
        }
    }

    test_inline!(
        Default::default(),
        |_| as_folder(TestVisitor {}),
        power_assert_recorder,
        // Input codes
        r#""#,
        // Output codes after transformed with plugin
        r#"
class _powerAssertRecorder {
    captured = [];
    _capt(value, espath) {
        this.captured.push({ value, espath });
        return value;
    }
    _expr(value, source) {
        const capturedValues = this.captured;
        this.captured = [];
        return {
            powerAssertContext: { value, events: capturedValues },
            source,
        };
    }
}"#
    );
}
