const { compile } = require('../..');
const { runInNewContext } = require('node:vm');

describe('Error stack', () => {
    it('should handle error stack correctly', () => {
        const program = `
class x {
    constructor(shouldThrow = false) {
        if (shouldThrow) {
            throw new Error('Has to be thrown');
        }
    }
}

new x();
new x(true);
`;

        const compiled = compile(program, 'x.js');
        try {
            runInNewContext(
                compiled,
                { Symbol, __jymfony, __jymfony_reflect, _apply_decs_2203_r },
                { filename: 'x.js' },
            );
            throw new Error('FAIL');
        } catch (e) {
            expect(e.stack).toContain(`x.js:11
            throw new Error('Has to be thrown');
            ^

Has to be thrown

    at new x (x.js:5:19)
    at x.js:11:1`);
        }
    });

    it('should read and rewrite multiple source map and handle error stack correctly', () => {
        const program = `
function x(shouldThrow = false) {
    if (shouldThrow) {
        throw new Error('Has to be thrown');
    }
}

new x();
new x(true);
`;

        const compiled = compile(program, 'x.ts');
        const recompiled = compile(compiled, 'x.ts');

        try {
            runInNewContext(recompiled, { Symbol }, { filename: 'x.ts' });
            throw new Error('FAIL');
        } catch (e) {
            expect(e.stack).toContain(`x.ts:3
        throw new Error('Has to be thrown');
        ^

Has to be thrown

    at new x (x.ts:4:15)
    at x.ts:9:1`);
        }
    });

    it('should compile exports correctly', () => {
        const program = `
export { x, y, z as ɵZ };
`;

        const compiled = compile(program, null);
        expect(compiled).toEqual(`"use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
function _export(target, all) {
    for(var name in all)Object.defineProperty(target, name, {
        enumerable: true,
        get: all[name]
    });
}
_export(exports, {
    x: function() {
        return x;
    },
    y: function() {
        return y;
    },
    ɵZ: function() {
        return z;
    }
});
`);
    });
});
