const { compile } = require('../..');
const { runInThisContext } = require('node:vm');

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

        const compiled = compile(program, 'x.js', { asFunction: true });
        try {
            runInThisContext(compiled, { filename: 'x.js' })();
            throw new Error('FAIL');
        } catch (e) {
            expect(e.stack).toContain(`Has to be thrown

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
        const recompiled = compile(compiled, 'x.ts', { asFunction: true });

        try {
            runInThisContext(recompiled, { filename: 'x.ts' })();
            throw new Error('FAIL');
        } catch (e) {
            expect(e.stack).toContain(`Has to be thrown

    at new x (x.ts:4:15)
    at x.ts:9:1`);
        }
    });
});
