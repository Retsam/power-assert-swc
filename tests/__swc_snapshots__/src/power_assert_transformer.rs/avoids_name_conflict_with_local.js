import { assert } from 'assert';
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
const _powerAssertRecorder1 = "name taken";
assert(_rec._expr(_rec._capt(true, "arguments/0"), {
    content: "assert(true)",
    filepath: "test.js",
    line: 3
}));
function f() {
    var _rec = new _powerAssertRecorder();
    assert(_rec._expr(_rec._capt(_powerAssertRecorder1, "arguments/0"), {
        content: "assert(_powerAssertRecorder)",
        filepath: "test.js",
        line: 5
    }));
}
