const { getInternalReflectionData } = require('..')._isSimdSupported
    ? require('../simd/compiler')
    : require('../pkg/compiler');

const reflectionDataMap = new Map();
const reflectionSymbol = Symbol('jymfony.reflect');

function docblockGetter(classId, memberIndex) {
    const data = getInternalReflectionData(classId);
    if (data === void 0) {
        return null;
    }

    const member = data.members[memberIndex];

    return member.docblock || null;
}

/**
 * @param {string} classId
 * @param {number|undefined} memberIndex
 */
exports._ = function __jymfony_reflect(classId, memberIndex = undefined) {
    return (value, context) => {
        const c = (() => {
            const d = reflectionDataMap.get(classId);
            if (d !== undefined) {
                return d;
            }

            const data = getInternalReflectionData(classId);
            if (data === void 0) {
                return {
                    fqcn: context.name,
                    className: context.name,
                    members: [],
                };
            }

            const c = { ...data };
            c.members = [];

            return c;
        })();

        if (context.kind === 'class') {
            context.metadata[reflectionSymbol] = classId;
            if (memberIndex !== void 0) {
                c.members.push({
                    memberIndex,
                    kind: 'method',
                    name: 'constructor',
                    static: false,
                    private: false,
                    access: { get: () => value },
                    get docblock() {
                        return docblockGetter(classId, memberIndex);
                    },
                    get parameters() {
                        const data = getInternalReflectionData(classId);
                        if (data === void 0) {
                            return [];
                        }

                        const member = data.members[memberIndex];

                        return member.params.map((p) => ({ ...p }));
                    },
                });
            }

            reflectionDataMap.set(classId, c);
            return;
        }

        if (!context.name) {
            return;
        }

        if (
            context.kind === 'method' ||
            context.kind === 'getter' ||
            context.kind === 'setter'
        ) {
            c.members.push({
                memberIndex,
                kind: context.kind,
                name: context.name,
                static: context.static,
                private: context.private,
                access: context.access,
                get parameters() {
                    const data = getInternalReflectionData(classId);
                    if (data === void 0) {
                        return [];
                    }

                    const member = data.members[memberIndex];

                    return member.params.map((p) => ({ ...p }));
                },
                get docblock() {
                    return docblockGetter(classId, memberIndex);
                },
            });
        }

        if (context.kind === 'field' || context.kind === 'accessor') {
            c.members.push({
                memberIndex,
                kind: context.kind,
                name: context.name,
                static: context.static,
                private: context.private,
                access: context.access,
                get docblock() {
                    return docblockGetter(classId, memberIndex);
                },
            });
        }

        reflectionDataMap.set(classId, c);
    };
};

exports.getReflectionData = function getReflectionData(classIdOrValue) {
    if (classIdOrValue === void 0 || classIdOrValue === null) {
        return undefined;
    }

    const metadata =
        classIdOrValue[Symbol.metadata || Symbol.for('Symbol.metadata')];
    if (metadata !== void 0 && metadata[reflectionSymbol] !== void 0) {
        classIdOrValue = metadata[reflectionSymbol];
    }

    return reflectionDataMap.get(classIdOrValue);
};
