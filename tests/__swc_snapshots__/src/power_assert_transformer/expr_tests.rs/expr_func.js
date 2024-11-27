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
assert(_rec._expr(_rec._capt(function() {
    return x == y;
}(), "arguments/0"), {
    content: "assert(function() { return x == y}())",
    filepath: "input/test.js",
    line: 3
}));
assert(_rec._expr(_rec._capt((()=>x == y)(), "arguments/0"), {
    content: "assert((() => x == y)())",
    filepath: "input/test.js",
    line: 4
}));
