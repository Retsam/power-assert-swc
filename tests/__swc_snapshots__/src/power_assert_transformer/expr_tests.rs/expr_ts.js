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
assert(_rec._expr(_rec._capt(_rec._capt(x, "arguments/0/left") == "a", "arguments/0") as false, {
    content: 'assert((x == "a") as false)',
    filepath: "input/test.ts",
    line: 3
}));
assert(_rec._expr(_rec._capt(_rec._capt(x, "arguments/0/left") == "b", "arguments/0") as const, {
    content: 'assert((x == "b") as const)',
    filepath: "input/test.ts",
    line: 4
}));
assert(_rec._expr(<false>_rec._capt(_rec._capt(x, "arguments/0/left") == "c", "arguments/0"), {
    content: 'assert(<false> (x == "c"))',
    filepath: "input/test.ts",
    line: 5
}));
assert(_rec._expr(_rec._capt(_rec._capt(x, "arguments/0/left")! == "d", "arguments/0"), {
    content: 'assert(x! == "d")',
    filepath: "input/test.ts",
    line: 6
}));
assert(_rec._expr(_rec._capt(_rec._capt(x, "arguments/0/left") == "e", "arguments/0") satisfies boolean, {
    content: 'assert((x == "e") satisfies boolean)',
    filepath: "input/test.ts",
    line: 7
}));
