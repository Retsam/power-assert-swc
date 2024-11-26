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
assert(_rec._expr(_rec._capt(_rec._capt({
    x,
    x: _rec._capt(y, "arguments/0/callee/object/properties/1/value"),
    func: function() {
        return this.x + 1;
    },
    meth () {
        return this.x + 2;
    },
    get x () {
        return this.x + 3;
    },
    set x (val){
        this.x = val;
    }
}, "arguments/0/callee/object").func(), "arguments/0"), {
    content: "assert({\n    x,\n    x: y,\n    func: function () {\n        return this.x + 1;\n    },\n    meth() {\n        return this.x + 2;\n    },\n    get x() {\n        return this.x + 3;\n    },\n    set x(val) {\n        this.x = val;\n    },\n}.func())",
    filepath: "input/test.js",
    line: 3
}));
