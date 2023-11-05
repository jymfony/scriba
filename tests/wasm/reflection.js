const { compile } = require('../..');
const { runInThisContext } = require('node:vm');

describe('Reflection', () => {
    it('should return class metadata', () => {
        const { getReflectionData } = require('../../lib/reflection');

        const program = `
Symbol.metadata = Symbol();
Symbol.parameters = Symbol();

const a = () => Symbol('test');
const type = t => {
    return function (value, context) {
        const metadata = context.metadata;
        metadata[context.function.name] = metadata[context.function.name] || {};
        metadata[context.function.name][Symbol.parameters] = metadata[context.function.name][Symbol.parameters] || {};
        metadata[context.function.name][Symbol.parameters][context.index] = metadata[context.function.name][Symbol.parameters][context.index] || {};
        metadata[context.function.name][Symbol.parameters][context.index].type = t;
    };
};

/** class docblock */
export default class x {
    static #staticPrivateField;
    #privateField;
    accessor #privateAccessor;
    static staticPublicField;
    publicField;
    accessor publicAccessor;
    
    /** constructor docblock */
    constructor(@type(String) constructorParam1) {
    }

    /**
     * computed method docblock
     */
    [a()]() {}
    #privateMethod(a, b = 1, [c, d], {f, g}) {}
    
    /**
     * public method docblock
     */
    publicMethod({a, b} = {}, c = new Object(), ...x) {}
    static #staticPrivateMethod() {}
    static staticPublicMethod() {}
    
    get [a()]() {}
    set b(v) {}
    
    get #ap() {}
    set #bp(v) {}
    
    act(@type(String) param1) {}
    [a()](@type(String) param1) {}
    [Symbol.for('xtest')](@type(String) param1) {}
    
    publicMethodWithDefaults(a = {}, b = 1, c = 'test', d = /test/g, e = 42n, f = true, g = null) {}
}

return x[Symbol.metadata].act[Symbol.parameters][0].type;
`;

        const compiled = compile(program, undefined, { debug: true });

        const exports = {};
        const t = runInThisContext(
            '(function(exports) {\n' + compiled + '\n})',
        )(exports);

        expect(t).toStrictEqual(String);

        const data = getReflectionData(exports['default']);
        const construct = data.members.find((o) => o.name === 'constructor');
        const member = data.members.find((o) => o.name === 'publicMethod');
        const defaults = data.members.find(
            (o) => o.name === 'publicMethodWithDefaults',
        );

        expect(construct).not.toBeUndefined();
        expect(construct.docblock).toEqual('/** constructor docblock */');
        expect(construct.parameters).toHaveLength(1);

        expect(member).not.toBeUndefined();
        expect(member.docblock).toEqual(
            '/**\n     * public method docblock\n     */',
        );
        expect(member.parameters).toHaveLength(3);
        expect(member.parameters[1].name).toEqual('c');
        expect(member.parameters[2].name).toEqual('x');
        expect(defaults.parameters).toHaveLength(7);
        expect(defaults.parameters[1].default).toEqual(1);
        expect(defaults.parameters[2].default).toEqual('test');
        expect(defaults.parameters[6].default).toEqual(null);
    });
});
