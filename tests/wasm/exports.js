const { compile } = require('../..');

describe('CommonJS', () => {
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
