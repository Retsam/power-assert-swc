use swc_core::{
    common::Mark,
    ecma::{
        ast::Pass,
        transforms::{
            base::resolver,
            testing::{test, Tester},
        },
        visit::visit_mut_pass,
    },
};
#[allow(unused)]
use swc_ecma_parser::Syntax;

use super::PowerAssertTransformerVisitor;

#[allow(unused)]
pub fn tr(tester: &mut Tester) -> impl Pass {
    (
        resolver(Mark::new(), Mark::new(), false),
        visit_mut_pass(PowerAssertTransformerVisitor::new(
            "input/test.js".into(),
            tester.cm.clone(),
        )),
    )
}
#[allow(unused)]
pub fn tr_ts(tester: &mut Tester) -> impl Pass {
    (
        resolver(Mark::new(), Mark::new(), false),
        visit_mut_pass(PowerAssertTransformerVisitor::new(
            "input/test.ts".into(),
            tester.cm.clone(),
        )),
    )
}

macro_rules! expr_test {
    ($name: ident, $code: literal) => {
        test!(
            Default::default(),
            tr,
            $name,
            &format!("import assert from 'assert';\n\n{}", $code)
        );
    };
}
macro_rules! expr_test_ts {
    ($name: ident, $code: literal) => {
        test!(
            Syntax::Typescript(Default::default()),
            tr_ts,
            $name,
            &format!("import assert from 'assert';\n\n{}", $code)
        );
    };
}
expr_test!(
    expr_basic,
    r#"
    assert(x.toUpperCase() == "BAR" ? x.y : x.z);
    "#
);

expr_test!(
    expr_assign,
    r#"
    assert(x = y.z);
    assert(a.b = y.z);
    assert((a.b.c = y.z = z));
    assert(x.y++ == y++);
    "#
);

expr_test!(
    expr_array,
    r#"
    assert([1, 2, 3]);
    assert([x, y])
    assert([1, 2, 3].find((el) => el > 3));
    "#
);

expr_test!(
    expr_classes,
    r#"
    assert(new x.y() instanceof SomeThing)

    assert(new (class foo {})());

    class Foo extends Bar {
        constructor() {
            super();
            assert(super.x)
        }
    }
    "#
);

expr_test!(
    expr_obj,
    r#"
assert({
    x,
    x: y,
    func: function () {
        return this.x + 1;
    },
    meth() {
        return this.x + 2;
    },
    get x() {
        return this.x + 3;
    },
    set x(val) {
        this.x = val;
    },
}.func());
"#
);

expr_test!(
    expr_func,
    r#"
assert(function() { return x == y}())
assert((() => x == y)())
"#
);
expr_test!(
    expr_yield_await,
    r#"
async function* foo() {
  assert(await (x, y));
  assert(yield (x, y));
}"#
);

expr_test!(
    expr_templates,
    r#"
assert(`${x},${y}`);
assert((x, y)`${y}${z}`);
"#
);
expr_test!(
    expr_comma,
    r#"
    assert((x, y))
    "#
);

expr_test!(
    expr_opt_chain,
    r#"
assert(x?.y.z == "a");
"#
);

expr_test!(
    expr_meta,
    r#"
class Foo {
  constructor() {
    assert(new.target.whatever);
  }
}
assert(import.meta.whatever)
"#
);

expr_test_ts!(
    expr_ts,
    r#"
assert((x == "a") as false);
assert((x == "b") as const);
assert(<false> (x == "c"));
assert(x! == "d");
assert((x == "e") satisfies boolean);
"#
);
