use swc_core::common::util::take::Take;
use swc_core::ecma::ast::*;

use super::power_assert_recorder::wrap_in_capture;
use super::PowerAssertTransformerVisitor;

#[derive(PartialEq)]
pub(super) enum CaptureExprContext {
    InsideCallee,
}

impl PowerAssertTransformerVisitor {
    /// This is the bulk of the logic for transforming expressions in order to capture their values
    pub(super) fn capture_expr(
        &mut self,
        mut node: Expr,
        path: Vec<String>,
        ctx: Option<CaptureExprContext>,
    ) -> Expr {
        let append_path = |str: &str| {
            let mut x = Vec::with_capacity(path.len() + 1);
            x.clone_from(&path);
            x.push(str.to_string());
            x
        };
        macro_rules! capt {
            ($self: ident, $expr: expr) => {{
                let e = $expr.into();
                if ctx == Some(CaptureExprContext::InsideCallee) {
                    e
                } else {
                    self.found_assertion = true;
                    self.needs_recorder = true;
                    wrap_in_capture(e, path.join("/"))
                }
            }};
        }

        /// This macro is a shortcut for the common pattern of recursing `capture_expr` into 'sub expressions', e.g. the left and right side of BinExpr
        macro_rules! capture_subs_exprs {
            ($self: ident, $orig: ident, $expr_kind: ident {
                $($field: ident $(as $path: expr)?),+
            }) => {
                $expr_kind { $($field: Box::new($self.capture_expr(*$orig.$field, append_path(path_from_field!($field $($path)?)), None))),+, ..$orig }
            };
        }
        macro_rules! path_from_field {
            ($field: ident) => {
                stringify!($field)
            };
            ($field: ident $path: expr) => {
                $path
            };
        }

        // Captures a single sub expr, with no change in the path
        macro_rules! capture_sub_expr {
            ($self: ident, $orig: ident.$field: ident) => {{
                let mut new_expr = $orig.clone();
                *new_expr.$field = $self.capture_expr(*$orig.$field, path.clone(), None);
                new_expr.into()
            }};
        }
        macro_rules! capture_sub_expr_with_path {
            ($self: ident, $orig: ident.$field: ident, $path: expr) => {{
                let mut new_expr = $orig.clone();
                *new_expr.$field = $self.capture_expr(*$orig.$field, append_path($path), None);
                Expr::from(new_expr)
            }};
        }

        // Common logic shared between Expr::Call and Expr::New
        macro_rules! capture_args {
            ($self: ident, $args: expr) => {
                $args
                    .into_iter()
                    .enumerate()
                    .map(|(i, arg)| capture_subs_exprs!(self, arg, ExprOrSpread {
                        expr as &format!("arguments/{i}")
                    }))
                    .collect::<Vec<_>>()
            };
        }

        let mut capture_template = |tpl: Tpl, prefix: &str| Tpl {
            exprs: tpl
                .exprs
                .into_iter()
                .enumerate()
                .map(|(i, expr)| {
                    Box::new(self.capture_expr(
                        *expr,
                        append_path(&format!("{prefix}expressions/{i}")),
                        None,
                    ))
                })
                .collect(),
            ..tpl
        };

        match node.take() {
            Expr::Array(array_lit) => capt!(self, ArrayLit {
                elems: array_lit.elems.into_iter()
                .enumerate()
                .map(|(i, maybe_el)| maybe_el.map(|el| capture_subs_exprs!(self, el, ExprOrSpread { expr as &format!("elements/{i}") }))).collect(),
                ..array_lit
            }),
            Expr::Object(object_lit) => {
                capt!(self, ObjectLit {
                    props: object_lit.props.into_iter()
                    .enumerate()
                    .map(|(i, prop)| {
                        if let PropOrSpread::Prop(inner_prop) = prop {
                            if let Prop::KeyValue(kv) = *inner_prop {
                                let key = if let Some(computed) = kv.key.as_computed().cloned() {
                                    capture_subs_exprs!(self, computed, ComputedPropName {
                                        expr as &format!("properties/{i}/key")
                                    }).into()
                                } else {
                                    kv.key
                                };
                                Prop::KeyValue(KeyValueProp {
                                    key,
                                    value: Box::new(self.capture_expr(*kv.value, append_path(&format!("properties/{i}/value")), None)),
                                })
                                .into()
                            } else { inner_prop.into() }
                        } else { prop }
                    }).collect(),
                    ..object_lit
                })
            },
            Expr::Unary(unary_expr) => capt!(
                self,
                capture_subs_exprs!(self, unary_expr, UnaryExpr { arg })
            ),
            Expr::Bin(bin_expr) => capt!(
                self,
                capture_subs_exprs!(self, bin_expr, BinExpr { left, right })
            ),
            Expr::Assign(assign_expr) => capt!(self, capture_subs_exprs!(self, assign_expr, AssignExpr { right })),
            Expr::Member(member_expr) => capt!(
                self,
                capture_subs_exprs!(self, member_expr, MemberExpr { obj as "object" })
            ),
            Expr::Cond(cond_expr) => {
                capture_subs_exprs!(self, cond_expr, CondExpr { test, cons as "consequent", alt as "alternate" })
                    .into()
            }
            Expr::Call(call_expr) => capt!(
                self,
                CallExpr {
                    callee: match call_expr.callee {
                        Callee::Expr(expr) => {
                            // We call self.capture_expr here, but not capt! - we don't necessarily capture the callee, but might capture parts of it
                            Callee::Expr(Box::new(self.capture_expr(
                                *expr,
                                append_path("callee"),
                                // We don't directly capture the callee (it's a function, not interesting to print out)
                                //  ... but there might be sub-expressions that are capture-able
                                // This flag will skip the capture in the recursive call
                                Some(CaptureExprContext::InsideCallee),
                            )))
                        }
                        // Nothing to capture in these cases
                        Callee::Super(_) | Callee::Import(_) => call_expr.callee,
                    },
                    args: capture_args!(self, call_expr.args),
                    ..call_expr
                }
            ),
            // Same as Expr::Call, except callee only has the Expr case and args is Optional
            Expr::New(new_expr) => capt!(
                self,
                NewExpr {
                    callee: Box::new(self.capture_expr(*new_expr.callee, append_path("callee"), Some(CaptureExprContext::InsideCallee))),
                    args: new_expr.args.map(|args| capture_args!(self, args)),
                    ..new_expr
                }
            ),
            Expr::Seq(seq_expr) => SeqExpr {
                exprs: seq_expr.exprs.into_iter()
                .enumerate()
                .map(|(i, expr)| Box::new(self.capture_expr(*expr, append_path(&format!("expressions/{i}")), None))).collect(),
                ..seq_expr
            }.into(),
            Expr::Tpl(tpl) => capt!(self, capture_template(tpl, "")),
            Expr::TaggedTpl(tagged_tpl) => {
                let tpl = capture_template(*tagged_tpl.tpl, "quasi/").into();
                let tag = self.capture_expr(*tagged_tpl.tag, append_path("tag"), None).into();
                capt!(self, TaggedTpl { tag, tpl, ..tagged_tpl })
            },
            expr @ (Expr::Ident(_) | Expr::SuperProp(_) | Expr::Update(_)) => capt!(self, expr),

            Expr::Yield(yield_expr) => capt!(self, YieldExpr {
                arg: yield_expr.arg.map(|arg| Box::new(self.capture_expr(*arg, append_path("argument"), None))),
                ..yield_expr
            }),
            Expr::Await(await_expr) => capt!(self, capture_sub_expr_with_path!(self, await_expr.arg, "argument")),

            // "Boring" cases where the expr is just a thin wrapper and we just want to recurse into the wrapper
            Expr::Paren(paren_expr) => capture_sub_expr!(self, paren_expr.expr),
            Expr::TsAs(ts_as_expr) => capture_sub_expr!(self, ts_as_expr.expr),
            Expr::TsTypeAssertion(ts_type_assertion) => capture_sub_expr!(self, ts_type_assertion.expr),
            Expr::TsConstAssertion(ts_const_assertion) => capture_sub_expr!(self, ts_const_assertion.expr),
            Expr::TsInstantiation(ts_instantiation) => capture_sub_expr!(self, ts_instantiation.expr),
            Expr::TsNonNull(ts_non_null_expr) => capture_sub_expr!(self, ts_non_null_expr.expr),
            Expr::TsSatisfies(ts_satisfies_expr) => capture_sub_expr!(self, ts_satisfies_expr.expr),

            // Exprs that are just ignored, neither captured nor recursed into
            expr @ (
                Expr::Lit(_) | Expr::This(_) | Expr::Class(_) | Expr::Fn(_) | Expr::Arrow(_) | Expr::Invalid(_) | Expr::PrivateName(_) | Expr::MetaProp(_) |
                Expr::JSXMember(_) | Expr::JSXNamespacedName(_) | Expr::JSXEmpty(_) | Expr::JSXElement(_) | Expr::JSXFragment(_)
            ) => expr,
            // Probably an opportunity for improvement here - the babel plugin doesn't transform this, but probably this syntax is just newer than power-assert
            expr @ Expr::OptChain(_) => expr,
        }
    }
}
