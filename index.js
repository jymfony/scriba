/* From wasm-feature-detect (https://www.npmjs.com/package/wasm-feature-detect) */
const isSimdSupported = WebAssembly.validate(
    new Uint8Array([
        0, 97, 115, 109, 1, 0, 0, 0, 1, 5, 1, 96, 0, 1, 123, 3, 2, 1, 0, 10, 10,
        1, 8, 0, 65, 0, 253, 15, 253, 98, 11,
    ]),
);
const { compile, prepareStackTrace, start } = isSimdSupported
    ? require('./simd/compiler')
    : require('./pkg/compiler');

exports._isSimdSupported = isSimdSupported;
exports.compile = compile;
exports.prepareStackTrace = prepareStackTrace;
exports.start = start;
exports.getReflectionData = require('./lib/reflection').getReflectionData;

global._apply_decs_2203_r = require('./lib/_apply_decs_2203_r')._;
global.__jymfony_reflect = require('./lib/reflection')._;
