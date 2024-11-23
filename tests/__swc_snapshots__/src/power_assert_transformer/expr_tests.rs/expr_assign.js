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
assert(_rec._expr(_rec._capt(x = _rec._capt(_rec._capt(y, "arguments/0/right/object").z, "arguments/0/right"), "arguments/0"), {
    content: "assert(x = y.z)",
    filepath: "input/test.js",
    line: 3
}));
assert(_rec._expr(_rec._capt(a.b = _rec._capt(_rec._capt(y, "arguments/0/right/object").z, "arguments/0/right"), "arguments/0"), {
    content: "assert(a.b = y.z)",
    filepath: "input/test.js",
    line: 4
}));
assert(_rec._expr(_rec._capt(a.b.c = _rec._capt(y.z = _rec._capt(z, "arguments/0/right/right"), "arguments/0/right"), "arguments/0"), {
    content: "assert((a.b.c = y.z = z))",
    filepath: "input/test.js",
    line: 5
}));
