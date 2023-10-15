const sym = Symbol();

function dec(_, ctx) {
  expect(ctx.function.kind).toEqual('method');
  expect(ctx.function.name).toEqual(sym);
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
  [sym](@dec [a]) {}
}

expect(A[Symbol.metadata][sym][Symbol.parameters]).toEqual({ 0: { rest: false, name: undefined, foo: 5 } });
