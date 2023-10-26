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

export default /** class docblock */ class x {
    static #staticPrivateField;
    #privateField;
    accessor #privateAccessor;
    static staticPublicField;
    publicField;
    accessor publicAccessor;

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
        const member = data.members.find((o) => o.name === 'publicMethod');

        expect(member).not.toBeUndefined();
        expect(member.docblock).toEqual(
            '/**\n     * public method docblock\n     */',
        );
        expect(member.parameters).toHaveLength(3);
    });
});
