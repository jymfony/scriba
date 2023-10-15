function dec(_, ctx) {
  expect(ctx.function.kind).toEqual('method');
  expect(ctx.function.name).toEqual('testMethod');
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
  testMethod(@dec ...a) {}
}

expect(A[Symbol.metadata].testMethod[Symbol.parameters]).toEqual({ 0: { rest: true, name: 'a', foo: 5 } });
