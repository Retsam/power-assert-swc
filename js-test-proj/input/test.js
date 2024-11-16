const assert = require("assert");

test("jest test test", () => {
  let x = "foo";
  assert(x.toUpperCase() === "BAR");
  notAssert(x.toUpperCase() === "BAR");
});
