import { _powerAssertRecorder } from "somewhere-else";
import { assert } from 'assert';
class _powerAssertRecorder1 {
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
var _rec = new _powerAssertRecorder1();
assert(_rec._expr(_rec._capt(a, "arguments/0"), {
    content: "assert(a)",
    filepath: "input/test.js",
    line: 3
}));
function f() {
    var _rec = new _powerAssertRecorder1();
    assert(_rec._expr(_rec._capt(a, "arguments/0"), {
        content: "assert(a)",
        filepath: "input/test.js",
        line: 5
    }));
}
