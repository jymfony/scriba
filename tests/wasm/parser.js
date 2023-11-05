const { isValidIdentifier } = require('../..');

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
});
