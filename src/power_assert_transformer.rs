use swc_core::ecma::{ast::*, visit::VisitMut};

mod power_assert_recorder;

/// This is the equivalent of babel-plugin-espower - transforms plain assertion calls into power-assert calls
pub struct PowerAssertTransformerVisitor;

impl VisitMut for PowerAssertTransformerVisitor {
    fn visit_mut_call_expr(&mut self, node: &mut CallExpr) {
        let inner = || {
            let callee = node.callee.as_expr()?;
            match &**callee {
                // Could probably use a COW here
                Expr::Ident(id) => Some(id.sym.to_string()),
                Expr::Member(mem) => full_member_expr_name(mem),
                _ => None,
            }
        };
        inner();
    }
}

// Takes a member expr and flattens it out into a string, e.g. `"assert.eq"`
fn full_member_expr_name(mem: &MemberExpr) -> Option<String> {
    let mut parts: Vec<&str> = vec![mem.prop.as_ident()?.sym.as_str()];
    let mut lhs = &*mem.obj;
    loop {
        match lhs {
            Expr::Ident(id) => {
                parts.push(id.sym.as_str());
                break;
            }
            Expr::Member(mem) => {
                parts.push(mem.prop.as_ident()?.sym.as_str());
                lhs = &*mem.obj;
            }
            _ => return None,
        }
    }
    Some(parts.into_iter().rev().collect::<Vec<_>>().join("."))
}
