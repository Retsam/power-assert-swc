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
class Foo {
    constructor() {
        var _rec = new _powerAssertRecorder();
        assert(_rec._expr(_rec._capt(new.target.whatever, "arguments/0"), {
            content: "assert(new.target.whatever)",
            filepath: "input/test.js",
            line: 5
        }));
    }
}
assert(_rec._expr(_rec._capt(import.meta.whatever, "arguments/0"), {
    content: "assert(import.meta.whatever)",
    filepath: "input/test.js",
    line: 8
}));
