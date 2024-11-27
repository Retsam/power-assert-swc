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
async function* foo() {
    var _rec = new _powerAssertRecorder();
    assert(_rec._expr(_rec._capt(await (_rec._capt(x, "arguments/0/argument/expressions/0"), _rec._capt(y, "arguments/0/argument/expressions/1")), "arguments/0"), {
        content: "assert(await (x, y))",
        filepath: "input/test.js",
        line: 4,
        async: true,
        generator: true
    }));
    assert(_rec._expr(_rec._capt((yield (_rec._capt(x, "arguments/0/argument/expressions/0"), _rec._capt(y, "arguments/0/argument/expressions/1"))), "arguments/0"), {
        content: "assert(yield (x, y))",
        filepath: "input/test.js",
        line: 5,
        async: true,
        generator: true
    }));
}
