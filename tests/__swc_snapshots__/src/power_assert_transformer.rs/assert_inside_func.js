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
function f1() {
    var _rec = new _powerAssertRecorder();
    assert(_rec.capt(true));
}
// expr
const f2 = function foo() {
    var _rec = new _powerAssertRecorder();
    assert(_rec.capt(true));
};
// arrow
const f3 = ()=>{
    var _rec = new _powerAssertRecorder();
    assert(_rec.capt(true));
};
// arrow shorthand
const f4 = ()=>{
    var _rec = new _powerAssertRecorder();
    assert(_rec.capt(true));
};
// nested
function outer() {
    var _rec = new _powerAssertRecorder();
    assert(_rec.capt(true));
    function inner() {
        var _rec = new _powerAssertRecorder();
        assert(_rec.capt(true));
    }
}
