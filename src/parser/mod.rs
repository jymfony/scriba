use crate::SyntaxError;
use anyhow::Result;
pub use program::CompileOptions;
use program::Program;
use std::path::PathBuf;
use std::rc::Rc;
use swc_common::comments::SingleThreadedComments;
use swc_common::input::StringInput;
use swc_common::sync::Lrc;
use swc_common::FileName;
use swc_ecma_ast::EsVersion;
use swc_ecma_parser::lexer::Lexer;
use swc_ecma_parser::{EsConfig, Parser, Syntax, TsConfig};

mod program;
mod sourcemap;
mod transformers;
mod util;

pub const ES_VERSION: EsVersion = EsVersion::EsNext;

pub fn parse(code: String, filename: Option<&str>) -> Result<Program> {
    let source_map: Lrc<swc_common::SourceMap> = Default::default();
    let source_file = source_map.new_source_file(
        filename
            .map(|f| FileName::Real(PathBuf::from(f)))
            .unwrap_or_else(|| FileName::Anon),
        code,
    );

    let comments = SingleThreadedComments::default();
    let is_typescript = filename.is_some_and(|f| f.ends_with(".ts"));
    let syntax = if is_typescript {
        Syntax::Typescript(TsConfig {
            tsx: false,
            decorators: true,
            dts: false,
            no_early_errors: false,
            disallow_ambiguous_jsx_like: false,
        })
    } else {
        Syntax::Es(EsConfig {
            jsx: false,
            fn_bind: false,
            decorators: true,
            decorators_before_export: false,
            export_default_from: false,
            import_attributes: false,
            allow_super_outside_method: false,
            allow_return_outside_function: true,
            auto_accessors: true,
            explicit_resource_management: true,
        })
    };

    let lexer = Lexer::new(
        syntax,
        ES_VERSION,
        StringInput::from(&*source_file),
        Some(&comments),
    );

    let mut parser = Parser::new_from(lexer);
    let parse_result = parser.parse_program();
    let orig_srcmap = sourcemap::get_orig_src_map(&source_file).unwrap_or_default();

    if let Ok(program) = parse_result {
        let errors = parser.take_errors();
        if !errors.is_empty() {
            let e = errors.first().unwrap();
            Err(SyntaxError::from_parser_error(e, &source_file).into())
        } else {
            Ok(Program {
                source_map,
                orig_srcmap,
                filename: filename.map(|f| f.to_string()),
                program,
                comments: Rc::new(comments),
                is_typescript,
            })
        }
    } else {
        let e = parse_result.unwrap_err();
        Err(SyntaxError::from_parser_error(&e, &source_file).into())
    }
}

#[cfg(test)]
mod tests {
    use super::parse;
    use crate::parser::transformers::decorator_2022_03;
    use crate::testing::exec_tr;
    use crate::testing::uuid::reset_test_uuid;
    use std::path::PathBuf;
    use swc_common::{chain, Mark};
    use swc_ecma_parser::{EsConfig, Syntax};
    use swc_ecma_transforms_base::resolver;
    use swc_ecma_transforms_compat::es2022::static_blocks;
    use swc_ecma_visit::Fold;

    #[testing::fixture("tests/decorators/**/exec.js")]
    pub fn exec(input: PathBuf) {
        exec_inner(input);
    }

    fn exec_inner(input: PathBuf) {
        let code = std::fs::read_to_string(&input).unwrap();

        exec_tr(
            "decorator",
            Syntax::Es(EsConfig {
                jsx: false,
                fn_bind: false,
                decorators: true,
                decorators_before_export: false,
                export_default_from: false,
                import_attributes: false,
                allow_super_outside_method: false,
                allow_return_outside_function: true,
                auto_accessors: true,
                explicit_resource_management: true,
            }),
            |_| create_pass(),
            &code,
        );
    }

    fn create_pass() -> Box<dyn Fold> {
        let unresolved_mark = Mark::new();
        let top_level_mark = Mark::new();
        let static_block_mark = Mark::new();

        Box::new(chain!(
            resolver(unresolved_mark, top_level_mark, false),
            decorator_2022_03(),
            static_blocks(static_block_mark),
        ))
    }

    #[test]
    pub fn test_resolve_self_identifiers() {
        reset_test_uuid();

        let code = r#"
const p = class {
    t() {
        return __self.x;
    }
};

export class x {
    m() {
        return __self.y;
    }
}

export class y {
    f() {
        return new class {
            c() {
                return __self;
            }
        }
    }
}
"#;

        let result = parse(code.into(), None).expect("failed to parse");
        let compiled = result
            .compile(Default::default())
            .expect("failed to compile");

        assert_eq!(
            compiled,
            r#""use strict";
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
        return _x;
    },
    y: function() {
        return _y;
    }
});
var _initClass, __anonymous_xΞ1, _dec, __jymfony_JObject, _dec1, _initProto, _dec2, _initClass1, __jymfony_JObject1, _dec3, _initProto1, _dec4, _initClass2, __jymfony_JObject2, _dec5, _initProto2;
_dec = __jymfony_reflect("00000000-0000-0000-0000-000000000000"), _dec1 = __jymfony_reflect("00000000-0000-0000-0000-000000000000", 0);
const p = (class _anonymous_xΞ1 extends (__jymfony_JObject = __jymfony.JObject) {
    static #_ = { e: [_initProto], c: [__anonymous_xΞ1, _initClass] } = _apply_decs_2203_r(this, [
        [
            _dec1,
            2,
            "t"
        ]
    ], [
        _dec
    ], __jymfony_JObject);
    constructor(...args){
        super(...args);
        _initProto(this);
    }
    t() {
        return __anonymous_xΞ1.x;
    }
    static #_2 = _initClass();
}, __anonymous_xΞ1);
let _x;
_dec2 = __jymfony_reflect("00000000-0000-0000-0000-000000000001"), _dec3 = __jymfony_reflect("00000000-0000-0000-0000-000000000001", 0);
class x extends (__jymfony_JObject1 = __jymfony.JObject) {
    static #_ = { e: [_initProto1], c: [_x, _initClass1] } = _apply_decs_2203_r(this, [
        [
            _dec3,
            2,
            "m"
        ]
    ], [
        _dec2
    ], __jymfony_JObject1);
    constructor(...args){
        super(...args);
        _initProto1(this);
    }
    m() {
        return _x.y;
    }
    static #_2 = _initClass1();
}
let _y;
_dec4 = __jymfony_reflect("00000000-0000-0000-0000-000000000002"), _dec5 = __jymfony_reflect("00000000-0000-0000-0000-000000000002", 0);
class y extends (__jymfony_JObject2 = __jymfony.JObject) {
    static #_ = { e: [_initProto2], c: [_y, _initClass2] } = _apply_decs_2203_r(this, [
        [
            _dec5,
            2,
            "f"
        ]
    ], [
        _dec4
    ], __jymfony_JObject2);
    constructor(...args){
        super(...args);
        _initProto2(this);
    }
    f() {
        var _initClass, __anonymous_xΞ2, _dec, __jymfony_JObject, _dec1, _initProto;
        _dec = __jymfony_reflect("00000000-0000-0000-0000-000000000003"), _dec1 = __jymfony_reflect("00000000-0000-0000-0000-000000000003", 0);
        return new (class _anonymous_xΞ2 extends (__jymfony_JObject = __jymfony.JObject) {
            static #_ = { e: [_initProto], c: [__anonymous_xΞ2, _initClass] } = _apply_decs_2203_r(this, [
                [
                    _dec1,
                    2,
                    "c"
                ]
            ], [
                _dec
            ], __jymfony_JObject);
            constructor(...args){
                super(...args);
                _initProto(this);
            }
            c() {
                return __anonymous_xΞ2;
            }
            static #_2 = _initClass();
        }, __anonymous_xΞ2);
    }
    static #_2 = _initClass2();
}
"#
        );
    }

    #[test]
    pub fn parse_error() {
        let code = r#"
new class ext impl test {[]}
"#;
        let result = parse(code.into(), Some("a.js"));
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert_eq!(
            error.to_string(),
            "SyntaxError: Expected '{', got 'impl' on line 2, column 15"
        );
    }

    #[test]
    pub fn parse_should_work() {
        let code = r#"
function register() { return () => {}; }
function initialize() { return () => {}; }
const secondary = () => console.log;
const logger = {
    logged: (value, { kind, name }) => {
        if (kind === "method") {
            return function (...args) {
                console.log(`starting \${name} with arguments \${args.join(", ")}`);
                const ret = value.call(this, ...args);
                console.log(`ending \${name}`);
                return ret;
            };
        }

        if (kind === "field") {
            return function (initialValue) {
                console.log(`initializing \${name} with value \${initialValue}`);
                return initialValue;
            };
        }
    },
}

const an = class {
    constructor() {
        // Dummy
        this.x = "test";
    }
};
const an1 = function () {};

// This is a comment
export default @logger.logged class x {
  @logger.logged
  @register((target, prop, parameterIndex = null) => {})
  @initialize((instance, key, value) => {})
  field = 'foo';

  @logger.logged
  @initialize((instance, key, value) => {})
  accessor fieldAcc = 'foobar';

  @logger.logged
  #privateField = 'pr';
  accessor #privateAccessor = 'acc';

  @logger.logged
  @secondary('great')
  test() {
    const cc = @logger.logged class {}
  }

  @logger.logged
  @secondary('great')
  get test_getter() {
    return 'test';
  }

  @logger.logged
  @secondary('great')
  set test_setter(value) {
  }

  @logger.logged
  testMethod(@type(Request) firstArg) {
    dump(firstArg);
  }

  @logger.logged
  testMethod2(@type(Request) ...[a, b,, c]) {
    dump(firstArg);
  }
}
"#;

        let parsed = parse(code.into(), Some("a.js")).unwrap();

        assert!(parsed.program.is_module());
        assert!(parsed.program.as_module().unwrap().body.iter().any(|s| s
            .as_module_decl()
            .is_some_and(|s| s.is_export_default_decl())));

        let _ = parsed
            .compile(Default::default())
            .expect("Should compile with no error");
    }
}
