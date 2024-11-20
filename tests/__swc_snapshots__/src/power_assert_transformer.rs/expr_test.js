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
assert(_rec._expr(_rec._capt(_rec._capt(_rec._capt(x, "arguments/0/left/callee/object").toUpperCase(), "arguments/0/left") == "BAR", "arguments/0"), {
    content: 'assert(x.toUpperCase() == "BAR")',
    filepath: "test.js",
    line: 3
}));
