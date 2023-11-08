const { isValidIdentifier, getArgumentNames } = require('../..');

describe('Parser', () => {
    const identifiers = ['x', 'y', 'ident'];
    const invalidIdentifiers = ['', null, 'abstract', 'x y'];

    for (const i of identifiers) {
        it('should validate js identifier: ' + JSON.stringify(i), () => {
            expect(isValidIdentifier(i)).toBeTruthy();
        });
    }

    for (const i of invalidIdentifiers) {
        it(
            'should validate invalid js identifiers: ' + JSON.stringify(i),
            () => {
                expect(isValidIdentifier(i)).toBeFalsy();
            },
        );
    }

    it('should return function parameters names', () => {
        const noArg = function () {};
        const arrArg = function ([a, b], args) {};
        const contextArg = function (context = {}) {};
        const restArg = function (...obj) {};
        const arrowContext = (context = {}) => {};
        const arrowNoParens = (arg) => arg;

        expect(getArgumentNames('function() {}')).toEqual([]);
        expect(getArgumentNames(noArg.toString())).toEqual([]);
        expect(getArgumentNames(arrArg.toString())).toEqual(['', 'args']);
        expect(getArgumentNames(contextArg.toString())).toEqual(['context']);
        expect(getArgumentNames(restArg.toString())).toEqual(['obj']);
        expect(getArgumentNames(arrowContext.toString())).toEqual(['context']);
        expect(getArgumentNames(arrowNoParens.toString())).toEqual(['arg']);

        expect(() => getArgumentNames('class x {}')).toThrowError();
        expect(() =>
            getArgumentNames('module.exports = function () {}'),
        ).toThrowError();
    });
});
