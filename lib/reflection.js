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
        const c = reflectionDataMap.get(classId) || { members: [] };
        if (context.kind === 'class') {
            context.metadata[reflectionSymbol] = classId;
            return;
        }

        if (!context.name) {
            return;
        }

        const data = getInternalReflectionData(classId);
        if (data === void 0) {
            return;
        }

        if (context.kind === 'method') {
            const getter = (() => {
                if (context.access) {
                    return context.access.get;
                }

                return () => value;
            })();

            c.members.push({
                memberIndex,
                kind: context.kind,
                name: context.name,
                static: context.static,
                private: context.private,
                access: { get: getter },
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
