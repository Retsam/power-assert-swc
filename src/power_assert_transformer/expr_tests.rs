use swc_core::ecma::transforms::testing::test;

#[allow(unused)]
use super::tr;

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
    expr_comma,
    r#"
    assert((x, y))
    "#
);
