function dec(_, ctx) {
    expect(ctx.function.kind).toEqual('class');
    expect(ctx.function.name).toBeUndefined();
    ctx.metadata[Symbol.parameters] = {};
    ctx.metadata[Symbol.parameters][ctx.index] = {
        rest: ctx.rest,
        name: ctx.name,
        foo: 5,
    };
}

Symbol.metadata = Symbol();
Symbol.parameters = Symbol();

class A {
    constructor(@dec a) {}
}

expect(A[Symbol.metadata][Symbol.parameters]).toEqual({
    0: { rest: false, name: 'a', foo: 5 },
});
expect(Object.getPrototypeOf(A[Symbol.metadata])).toBe(null);
