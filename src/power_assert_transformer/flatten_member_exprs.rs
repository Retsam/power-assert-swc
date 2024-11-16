use swc_core::{atoms::Atom, ecma::ast::Expr};

// Takes a expr and flattens out any member exprs into a single string, e.g. `"assert.eq"`
pub fn flatten_member_exprs(expr: &Expr) -> Option<Atom> {
    let mut parts: Vec<&Atom> = vec![];
    let mut lhs = expr;

    loop {
        match lhs {
            Expr::Ident(id) => {
                parts.push(&id.sym);
                break;
            }
            Expr::Member(mem) => {
                parts.push(&mem.prop.as_ident()?.sym);
                lhs = &*mem.obj;
            }
            _ => return None,
        }
    }
    // If it's a simple access, only one entry here, just return a clone of it (cloning Atom is cheap)
    if parts.len() == 1 {
        Some(parts[0].clone())
    } else {
        Some(
            parts
                .into_iter()
                .rev()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join(".")
                .into(),
        )
    }
}
