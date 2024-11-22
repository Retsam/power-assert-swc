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
class C {
    constructor(){
        var _rec = new _powerAssertRecorder();
        assert(_rec._expr(_rec._capt(a, "arguments/0"), {
            content: "assert(a)",
            filepath: "input/test.js",
            line: 3
        }));
    }
    m1() {
        var _rec = new _powerAssertRecorder();
        assert(_rec._expr(_rec._capt(a, "arguments/0"), {
            content: "assert(a)",
            filepath: "input/test.js",
            line: 6
        }));
    }
    #m2() {
        var _rec = new _powerAssertRecorder();
        assert(_rec._expr(_rec._capt(a, "arguments/0"), {
            content: "assert(a)",
            filepath: "input/test.js",
            line: 9
        }));
    }
}
