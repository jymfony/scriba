use crate::parser::transformers::{
    anonymous_expr, class_define_fields, class_jobject, class_reflection_decorators,
    decorator_2022_03, optional_import, remove_assert_calls, resolve_self_identifiers,
    static_blocks, wrap_in_function,
};
use crate::stack::register_source_map;
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use sourcemap::SourceMap;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use swc_cached::regex::CachedRegex;
use swc_common::comments::SingleThreadedComments;
use swc_common::sync::Lrc;
use swc_common::{chain, BytePos, LineCol, Mark, GLOBALS};
use swc_ecma_codegen::text_writer::JsWriter;
use swc_ecma_codegen::Emitter;
use swc_ecma_transforms_base::feature::FeatureFlag;
use swc_ecma_transforms_base::fixer::fixer;
use swc_ecma_transforms_base::helpers::{inject_helpers, Helpers, HELPERS};
use swc_ecma_transforms_base::hygiene::hygiene;
use swc_ecma_transforms_base::resolver;
use swc_ecma_transforms_compat::es2020::{nullish_coalescing, optional_chaining};
use swc_ecma_transforms_module::common_js;
use swc_ecma_transforms_module::util::{ImportInterop, Lazy, LazyObjectConfig};
use swc_ecma_transforms_typescript::strip;
use swc_ecma_visit::{Fold, FoldWith};

#[derive(Default)]
pub struct CompileOptions {
    pub debug: bool,
    pub namespace: Option<String>,
    pub as_function: bool,
}

pub struct Program {
    pub(crate) source_map: Lrc<swc_common::SourceMap>,
    pub(crate) orig_srcmap: Option<SourceMap>,
    pub(crate) filename: Option<String>,
    pub(crate) program: swc_ecma_ast::Program,
    pub(crate) comments: Rc<SingleThreadedComments>,
    pub(crate) is_typescript: bool,
}

impl Debug for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Program")
            .field("filename", &self.filename)
            .field("program", &self.program)
            .field("comments", &self.comments)
            .field("is_typescript", &self.is_typescript)
            .finish_non_exhaustive()
    }
}

impl Program {
    pub fn compile(self, opts: CompileOptions) -> std::io::Result<String> {
        GLOBALS.set(&Default::default(), || {
            let helpers = Helpers::new(false);
            HELPERS.set(&helpers, || {
                let unresolved_mark = Mark::new();
                let top_level_mark = Mark::new();
                let static_block_mark = Mark::new();
                let available_set = FeatureFlag::all();

                let common_js_config = common_js::Config {
                    import_interop: Some(ImportInterop::Swc),
                    lazy: Lazy::Object(LazyObjectConfig {
                        patterns: vec![
                            CachedRegex::new(".+").unwrap(),
                        ],
                    }),
                    ..Default::default()
                };

                let mut transformers: Box<dyn Fold> = Box::new(chain!(
                    resolver(unresolved_mark, top_level_mark, self.is_typescript),
                    anonymous_expr(),
                    class_reflection_decorators(
                        self.filename.as_deref(),
                        opts.namespace.as_deref(),
                        self.comments.clone()
                    ),
                    strip(top_level_mark),
                    optional_import(unresolved_mark),
                    nullish_coalescing(Default::default()),
                    optional_chaining(Default::default(), unresolved_mark),
                    resolve_self_identifiers(unresolved_mark),
                    class_jobject(),
                    decorator_2022_03(),
                    class_define_fields(),
                    static_blocks(static_block_mark),
                    common_js(
                        unresolved_mark,
                        common_js_config,
                        available_set,
                        Some(&self.comments)
                    ),
                    hygiene(),
                    fixer(Some(&self.comments)),
                    inject_helpers(top_level_mark),
                ));

                if !opts.debug {
                    transformers = Box::new(chain!(transformers, remove_assert_calls()));
                }

                if opts.as_function {
                    transformers = Box::new(chain!(transformers, wrap_in_function()));
                }

                let program = self.program.fold_with(transformers.as_mut());
                let mut buf = vec![];
                let mut sm: Vec<(BytePos, LineCol)> = vec![];

                {
                    let mut emitter = Emitter {
                        cfg: Default::default(),
                        cm: self.source_map.clone(),
                        comments: Some(&self.comments),
                        wr: JsWriter::new(Default::default(), "\n", &mut buf, Some(&mut sm)),
                    };

                    emitter.emit_program(&program)?
                };

                let mut src = String::from_utf8(buf).expect("non-utf8?");
                if let Some(f) = self.filename.as_deref() {
                    let srcmap = self
                        .source_map
                        .build_source_map_from(&sm, self.orig_srcmap.as_ref());

                    register_source_map(f.to_string(), srcmap.clone());

                    let mut buf = vec![];
                    srcmap.to_writer(&mut buf).ok();

                    let res = BASE64_STANDARD.encode(buf);
                    src += "\n\n//# sourceMappingURL=data:application/json;charset=utf-8;base64,";
                    src += &res;
                }

                Ok(src)
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::{CodeParser, CompileOptions};
    use crate::testing::uuid::reset_test_uuid;

    #[test]
    pub fn should_add_explicit_constructor_to_decorator_metadata() -> anyhow::Result<()> {
        reset_test_uuid();

        let code = r#"
export default class TestClass {
    constructor() {
        console.log(this);
    }
}
"#;
        let program = code.parse_program(None)?;
        let code = program.compile(CompileOptions {
            debug: false,
            namespace: None,
            as_function: false,
        })?;

        assert_eq!(
            code,
            r#""use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
var _initClass, _TestClass, _dec, __jymfony_JObject;
_dec = __jymfony_reflect("00000000-0000-0000-0000-000000000000", 0);
class TestClass extends (__jymfony_JObject = __jymfony.JObject) {
    static #_ = { c: [_TestClass, _initClass] } = _apply_decs_2203_r(this, [], [
        _dec
    ], __jymfony_JObject);
    constructor(){
        super();
        console.log(this);
    }
    static #_2 = _initClass();
}
const _default = _TestClass;
"#
        );

        Ok(())
    }

    #[test]
    pub fn compile_optional_imports_correctly() {
        reset_test_uuid();

        let code = r#"
import Redis from 'ioredis' with { optional: true };
import { parse as urlParse } from 'url';

const RedisCluster = Redis ? Redis.Cluster : undefined;
const parseHosts = (params, dsn) => {};

/**
 * @memberOf Jymfony.Component.Cache.Adapter
 */
export default class RedisAdapter {}
"#;
        let program = code.parse_program(None).unwrap();
        let code = program
            .compile(CompileOptions {
                debug: false,
                namespace: None,
                as_function: false,
            })
            .unwrap();

        assert_eq!(
            code,
            r#""use strict";
function _interop_require_default(obj) {
    return obj && obj.__esModule ? obj : {
        default: obj
    };
}
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, /**
 * @memberOf Jymfony.Component.Cache.Adapter
 */ "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
var _initClass, _RedisAdapter, _dec, __jymfony_JObject;
const _r = function() {
    try {
        return require("ioredis");
    } catch  {
        return void 0;
    }
}();
const Redis = void 0 !== _r ? _interop_require_default(_r, true).default : void 0;
const RedisCluster = Redis ? Redis.Cluster : undefined;
const parseHosts = (params, dsn)=>{};
_dec = __jymfony_reflect("00000000-0000-0000-0000-000000000000", void 0);
class RedisAdapter extends (__jymfony_JObject = __jymfony.JObject) {
    static #_ = { c: [_RedisAdapter, _initClass] } = _apply_decs_2203_r(this, [], [
        _dec
    ], __jymfony_JObject);
    static #_2 = _initClass();
}
const _default = _RedisAdapter;
"#
        );

        let code = r#"
import Redis, { Cluster as RedisCluster } from 'ioredis' with { optional: true };

class conn {
    constructor() {
        this._cluster = new RedisCluster();
        this._redis = Redis;
    }
}
"#;
        let program = code.parse_program(None).unwrap();
        let code = program
            .compile(CompileOptions {
                debug: false,
                namespace: None,
                as_function: false,
            })
            .unwrap();

        assert_eq!(
            code,
            r#""use strict";
function _interop_require_default(obj) {
    return obj && obj.__esModule ? obj : {
        default: obj
    };
}
var _dec, _initClass, __jymfony_JObject;
const _r = function() {
    try {
        return require("ioredis");
    } catch  {
        return void 0;
    }
}();
const Redis = void 0 !== _r ? _interop_require_default(_r, true).default : void 0;
const RedisCluster = _r === null || _r === void 0 ? void 0 : _r.Cluster;
let _conn;
_dec = __jymfony_reflect("00000000-0000-0000-0000-000000000001", 0);
class conn extends (__jymfony_JObject = __jymfony.JObject) {
    static #_ = { c: [_conn, _initClass] } = _apply_decs_2203_r(this, [], [
        _dec
    ], __jymfony_JObject);
    constructor(){
        super();
        this._cluster = new RedisCluster();
        this._redis = Redis;
    }
    static #_2 = _initClass();
}
"#
        );
    }

    #[test]
    pub fn should_compile_field_initialization() -> anyhow::Result<()> {
        reset_test_uuid();

        let program = r#"
const TestCase = Jymfony.Component.Testing.Framework.TestCase;

export default class ClassLoaderTest extends TestCase {
    /**
     * @type {Jymfony.Component.Autoloader.ClassLoader}
     */
    _classLoader;
}
"#
        .parse_program(None)?;

        let compiled = program.compile(CompileOptions {
            debug: false,
            namespace: None,
            as_function: false,
        })?;

        assert_eq!(
            compiled,
            r#""use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
var _initClass, _ClassLoaderTest, _dec, _TestCase, _dec1, /**
     * @type {Jymfony.Component.Autoloader.ClassLoader}
     */ _init__classLoader;
const TestCase = Jymfony.Component.Testing.Framework.TestCase;
_dec = __jymfony_reflect("00000000-0000-0000-0000-000000000000", void 0), _dec1 = __jymfony_reflect("00000000-0000-0000-0000-000000000000", 0);
class ClassLoaderTest extends (_TestCase = TestCase) {
    static #_ = { e: [_init__classLoader], c: [_ClassLoaderTest, _initClass] } = _apply_decs_2203_r(this, [
        [
            _dec1,
            0,
            "_classLoader"
        ]
    ], [
        _dec
    ], _TestCase);
    static #_2 = _initClass();
    [Symbol.__jymfony_field_initialization]() {
        const superCall = super[Symbol.__jymfony_field_initialization];
        if (void 0 !== superCall) superCall.apply(this);
        this._classLoader = _init__classLoader(this);
    }
}
const _default = _ClassLoaderTest;
"#
        );

        Ok(())
    }

    #[test]
    pub fn should_compile_imports_correctly() -> anyhow::Result<()> {
        reset_test_uuid();

        let program = r#"
import JsonFileLoaderTest from './JsonFileLoaderTest';
const YamlFileLoader = Jymfony.Component.Validator.Mapping.Loader.YamlFileLoader;

export default class YamlFileLoaderTest extends JsonFileLoaderTest {
}
"#
        .parse_program(None)?;

        let compiled = program.compile(CompileOptions {
            debug: false,
            namespace: None,
            as_function: false,
        })?;

        assert_eq!(
            compiled,
            r#""use strict";
function _interop_require_default(obj) {
    return obj && obj.__esModule ? obj : {
        default: obj
    };
}
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
function _JsonFileLoaderTest() {
    const data = /*#__PURE__*/ _interop_require_default(require("./JsonFileLoaderTest"));
    _JsonFileLoaderTest = function() {
        return data;
    };
    return data;
}
var _initClass, _YamlFileLoaderTest, _dec, _JsonFileLoaderTest1;
const YamlFileLoader = Jymfony.Component.Validator.Mapping.Loader.YamlFileLoader;
_dec = __jymfony_reflect("00000000-0000-0000-0000-000000000000", void 0);
class YamlFileLoaderTest extends (_JsonFileLoaderTest1 = _JsonFileLoaderTest().default) {
    static #_ = { c: [_YamlFileLoaderTest, _initClass] } = _apply_decs_2203_r(this, [], [
        _dec
    ], _JsonFileLoaderTest1);
    static #_2 = _initClass();
}
const _default = _YamlFileLoaderTest;
"#
        );

        Ok(())
    }

    #[test]
    pub fn should_compile_decorated_class() -> anyhow::Result<()> {
        reset_test_uuid();

        let program = r#"
const Annotation = Jymfony.Component.Autoloader.Decorator.Annotation;

/**
 * @memberOf Foo.Decorators
 */
export default
@Annotation()
class TestAnnotation {
    __construct(value) {
        this._value = value;
    }

    get value() {
        return this._value;
    }
}
"#
        .parse_program(None)?;

        let compiled = program.compile(CompileOptions {
            debug: false,
            namespace: None,
            as_function: false,
        })?;

        assert_eq!(
            compiled,
            r#""use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, /**
 * @memberOf Foo.Decorators
 */ "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
var _initClass, _TestAnnotation, _dec, _dec1, __jymfony_JObject, _dec2, _dec3, _initProto;
const Annotation = Jymfony.Component.Autoloader.Decorator.Annotation;
_dec = Annotation(), _dec1 = __jymfony_reflect("00000000-0000-0000-0000-000000000000", void 0), _dec2 = __jymfony_reflect("00000000-0000-0000-0000-000000000000", 0), _dec3 = __jymfony_reflect("00000000-0000-0000-0000-000000000000", 1);
class TestAnnotation extends (__jymfony_JObject = __jymfony.JObject) {
    static #_ = { e: [_initProto], c: [_TestAnnotation, _initClass] } = _apply_decs_2203_r(this, [
        [
            _dec2,
            2,
            "__construct"
        ],
        [
            _dec3,
            3,
            "value"
        ]
    ], [
        _dec,
        _dec1
    ], __jymfony_JObject);
    constructor(...args){
        super(...args);
        _initProto(this);
    }
    __construct(value) {
        this._value = value;
    }
    get value() {
        return this._value;
    }
    static #_2 = _initClass();
}
const _default = _TestAnnotation;
"#
        );

        let program = r#"
const BufferingLogger = Jymfony.Component.Debug.BufferingLogger;
const ErrorHandler = Jymfony.Component.Debug.ErrorHandler;
const Timeout = Jymfony.Component.Debug.Timeout;
const UnhandledRejectionException = Jymfony.Component.Debug.Exception.UnhandledRejectionException;

/**
 * @memberOf Jymfony.Component.Debug
 */
export default class Debug {
    static enable() {
        __jymfony.autoload.debug = true;

        process.on('unhandledRejection', (reason, p) => {
            throw new UnhandledRejectionException(p, reason instanceof Error ? reason : undefined);
        });

        __jymfony.ManagedProxy.enableDebug();
        Timeout.enable();
        ErrorHandler.register(new ErrorHandler(new BufferingLogger(), true));
    }
}
"#
        .parse_program(None)?;

        let compiled = program.compile(CompileOptions {
            debug: false,
            namespace: None,
            as_function: false,
        })?;

        assert_eq!(
            compiled,
            r#""use strict";
Object.defineProperty(exports, "__esModule", {
    value: true
});
Object.defineProperty(exports, /**
 * @memberOf Jymfony.Component.Debug
 */ "default", {
    enumerable: true,
    get: function() {
        return _default;
    }
});
var _initClass, _Debug, _dec, __jymfony_JObject, _dec1, _initStatic;
const BufferingLogger = Jymfony.Component.Debug.BufferingLogger;
const ErrorHandler = Jymfony.Component.Debug.ErrorHandler;
const Timeout = Jymfony.Component.Debug.Timeout;
const UnhandledRejectionException = Jymfony.Component.Debug.Exception.UnhandledRejectionException;
_dec = __jymfony_reflect("00000000-0000-0000-0000-000000000001", void 0), _dec1 = __jymfony_reflect("00000000-0000-0000-0000-000000000001", 0);
class Debug extends (__jymfony_JObject = __jymfony.JObject) {
    static #_ = (()=>{
        ({ e: [_initStatic], c: [_Debug, _initClass] } = _apply_decs_2203_r(this, [
            [
                _dec1,
                8,
                "enable"
            ]
        ], [
            _dec
        ], __jymfony_JObject));
        _initStatic(this);
    })();
    static enable() {
        __jymfony.autoload.debug = true;
        process.on('unhandledRejection', (reason, p)=>{
            throw new UnhandledRejectionException(p, reason instanceof Error ? reason : undefined);
        });
        __jymfony.ManagedProxy.enableDebug();
        Timeout.enable();
        ErrorHandler.register(new ErrorHandler(new BufferingLogger(), true));
    }
    static #_2 = _initClass();
}
const _default = _Debug;
"#
        );

        Ok(())
    }
}
