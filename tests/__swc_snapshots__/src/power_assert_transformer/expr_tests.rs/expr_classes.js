import assert from 'assert';
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
var _rec = new _powerAssertRecorder();
assert(_rec._expr(_rec._capt(_rec._capt(new (_rec._capt(x, "arguments/0/left/callee/object")).y(), "arguments/0/left") instanceof _rec._capt(SomeThing, "arguments/0/right"), "arguments/0"), {
    content: "assert(new x.y() instanceof SomeThing)",
    filepath: "input/test.js",
    line: 3
}));
assert(_rec._expr(_rec._capt(new class foo {
}(), "arguments/0"), {
    content: "assert(new (class foo {})())",
    filepath: "input/test.js",
    line: 5
}));
