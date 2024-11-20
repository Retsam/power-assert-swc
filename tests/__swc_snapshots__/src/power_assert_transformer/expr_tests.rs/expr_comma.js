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
assert(_rec._expr((_rec._capt(x, "arguments/0/expressions/0"), _rec._capt(y, "arguments/0/expressions/1")), {
    content: "assert((x, y))",
    filepath: "input/test.js",
    line: 3
}));
