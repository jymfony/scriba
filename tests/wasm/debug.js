const { compile } = require('../..');

describe('Debug', () => {
    it('should compile debug assertions correctly', () => {
        const program = `
function x() {
    __assert(0 === 1);
}
`;

        const compiled = compile(program, null, { debug: true });
        expect(compiled).toEqual(`function x() {
    __assert(0 === 1);
}
`);
    });

    it('should not compile debug assertions if debug flag is disabled', () => {
        const program = `
function x() {
    __assert(0 === 1);
}
`;

        const compiled = compile(program, null, { debug: false });
        expect(compiled).toEqual(`function x() {
    void 0;
}
`);
    });
});
