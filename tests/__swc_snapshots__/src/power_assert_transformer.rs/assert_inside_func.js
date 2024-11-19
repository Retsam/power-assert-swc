class _powerAssertRecorder {
    captured = [];
    _capt(value, espath) {
        this.captured.push({
            value,
            espath
        });
        return value;
    }
    _expr(value, source) {
        const capturedValues = this.captured;
        this.captured = [];
        return {
            powerAssertContext: {
                value,
                events: capturedValues
            },
            source
        };
    }
}
function f1() {
    var _rec = new _powerAssertRecorder();
    assert(_rec._expr(_rec._capt(a, "arguments/0"), {
        content: "assert(a)",
        filepath: "test.js",
        line: 2
    }));
}
// expr
const f2 = function foo() {
    var _rec = new _powerAssertRecorder();
    assert(_rec._expr(_rec._capt(a, "arguments/0"), {
        content: "assert(a)",
        filepath: "test.js",
        line: 6
    }));
};
// arrow
const f3 = ()=>{
    var _rec = new _powerAssertRecorder();
    assert(_rec._expr(_rec._capt(a, "arguments/0"), {
        content: "assert(a)",
        filepath: "test.js",
        line: 10
    }));
};
// arrow shorthand
const f4 = ()=>{
    var _rec = new _powerAssertRecorder();
    assert(_rec._expr(_rec._capt(a, "arguments/0"), {
        content: "assert(a)",
        filepath: "test.js",
        line: 13
    }));
};
// nested
function outer() {
    assert(true);
    function inner() {
        var _rec = new _powerAssertRecorder();
        assert(_rec._expr(_rec._capt(a, "arguments/0"), {
            content: "assert(a)",
            filepath: "test.js",
            line: 19
        }));
    }
}
