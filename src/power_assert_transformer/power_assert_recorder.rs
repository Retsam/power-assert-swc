use swc_core::{common::DUMMY_SP, ecma::ast::*};

const RECORDER_CLASS_NAME: &str = "_powerAssertRecorder";
const RECORDER_INSTANCE_NAME: &str = "_rec";
const CAPTURE_METHOD_NAME: &str = "_capt";
const EXPRESSION_WRAPPER_METHOD_NAME: &str = "_expr";

macro_rules! this_expr {
    () => {
        Box::new(Expr::This(ThisExpr { span: DUMMY_SP }))
    };
}
macro_rules! ident_name {
    ($sym: expr) => {
        IdentName {
            span: DUMMY_SP,
            sym: $sym.into(),
        }
        .into()
    };
}
macro_rules! from_ident {
    ($sym: expr) => {
        Into::<Ident>::into($sym).into()
    };
}

/// Returns the AST for the `class _powerAssertRecorder` which is injected into files that use power-assert
pub fn power_assert_recorder_definition() -> Stmt {
    let class = ClassDecl {
        ident: RECORDER_CLASS_NAME.into(),
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
                    key: from_ident!(CAPTURE_METHOD_NAME),
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
                    key: ident_name!(EXPRESSION_WRAPPER_METHOD_NAME),
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

pub fn new_power_assert_recorder_stmt() -> Stmt {
    Stmt::Decl(Decl::Var(Box::new(VarDecl {
        kind: VarDeclKind::Var, // TDZ has runtime cost
        decls: vec![VarDeclarator {
            span: DUMMY_SP,
            name: Into::<IdentName>::into(RECORDER_INSTANCE_NAME).into(),
            init: Some(
                NewExpr {
                    callee: Into::<Ident>::into(RECORDER_CLASS_NAME).into(),
                    // Put the empty parens on the new call - technically doesn't change the behavior but hurts my soul less
                    args: Some(vec![]),
                    ..Default::default()
                }
                .into(),
            ),
            definite: false,
        }],
        ..Default::default()
    })))
}

pub fn wrap_in_record(expr: Expr) -> Expr {
    Expr::Call(CallExpr {
        callee: Callee::Expr(Box::new(Expr::Member(MemberExpr {
            obj: Into::<Ident>::into("_rec").into(),
            prop: ident_name!(EXPRESSION_WRAPPER_METHOD_NAME),
            span: DUMMY_SP,
        }))),
        args: vec![ExprOrSpread {
            spread: None,
            expr: Box::new(expr),
        }],
        ..Default::default()
    })
}

pub fn wrap_in_capture(expr: Expr, path: String) -> Expr {
    Expr::Call(CallExpr {
        callee: Callee::Expr(Box::new(Expr::Member(MemberExpr {
            obj: Into::<Ident>::into("_rec").into(),
            prop: ident_name!(CAPTURE_METHOD_NAME),
            span: DUMMY_SP,
        }))),
        args: vec![
            ExprOrSpread {
                spread: None,
                expr: Box::new(expr),
            },
            ExprOrSpread {
                spread: None,
                expr: path.into(),
            },
        ],
        ..Default::default()
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use swc_core::ecma::{
        transforms::testing::test,
        visit::{visit_mut_pass, VisitMut},
    };

    struct TestVisitor;

    impl VisitMut for TestVisitor {
        fn visit_mut_script(&mut self, node: &mut Script) {
            node.body.push(power_assert_recorder_definition());
        }
    }

    test!(
        Default::default(),
        |_| visit_mut_pass(TestVisitor),
        power_assert_recorder,
        // Input codes
        r#""#
    );
}
