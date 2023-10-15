let calls = 0;

function generateMethodName() {
  return 'testMethod' + (++calls);
}

function dec(_, ctx) {
  expect(ctx.function.kind).toEqual('method');
  expect(ctx.function.name).toEqual('testMethod1');
  ctx.metadata[ctx.function.name] = {};
  ctx.metadata[ctx.function.name][Symbol.parameters] = {};
  ctx.metadata[ctx.function.name][Symbol.parameters][ctx.index] = {
    rest: ctx.rest,
    name: ctx.name,
    foo: 5,
  };
}

Symbol.metadata = Symbol();
Symbol.parameters = Symbol();

class A {
  [generateMethodName()](@dec a = 'x') {}
}

expect(calls).toBe(1);
expect(A[Symbol.metadata].testMethod1[Symbol.parameters]).toEqual({ 0: { rest: false, name: undefined, foo: 5 } });
