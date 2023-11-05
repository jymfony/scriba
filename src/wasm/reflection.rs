use crate::parse_uuid;
use crate::reflection::{get_reflection_data, ReflectionData};
use serde::{Deserialize, Serialize};
use swc_ecma_ast::*;
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum Scalar {
    Str(String),
    Bool(bool),
    Null,
    Num(f64),
    BigInt(String),
    Regex { exp: String, flags: String },
}

impl TryFrom<&Lit> for Scalar {
    type Error = ();

    fn try_from(value: &Lit) -> Result<Self, Self::Error> {
        match value {
            Lit::Str(s) => Ok(Scalar::Str(s.value.to_string())),
            Lit::Bool(b) => Ok(Scalar::Bool(b.value)),
            Lit::Null(_) => Ok(Scalar::Null),
            Lit::Num(n) => Ok(Scalar::Num(n.value)),
            Lit::BigInt(n) => Ok(Scalar::BigInt(n.value.to_string())),
            Lit::Regex(r) => Ok(Scalar::Regex {
                exp: r.exp.to_string(),
                flags: r.flags.to_string(),
            }),
            _ => Err(()),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsMethodParameter {
    pub name: Option<String>,
    pub index: usize,
    pub has_default: bool,
    pub scalar_default: Option<Scalar>,
    pub is_object_pattern: bool,
    pub is_array_pattern: bool,
    pub is_rest_element: bool,
}

impl From<&Param> for JsMethodParameter {
    fn from(value: &Param) -> Self {
        let (is_rest, pat) = if let Pat::Rest(r) = &value.pat {
            (true, r.arg.clone())
        } else {
            (false, Box::new(value.pat.clone()))
        };

        let (ident, def) = if let Pat::Assign(a) = pat.as_ref() {
            let def = a.right.as_lit().and_then(|l| Scalar::try_from(l).ok());
            (a.left.as_ident(), def)
        } else {
            (pat.as_ident(), None)
        };

        JsMethodParameter {
            name: ident.map(|i| i.sym.to_string()),
            index: 0,
            has_default: pat.is_assign(),
            scalar_default: def,
            is_object_pattern: pat.is_object(),
            is_array_pattern: pat.is_array(),
            is_rest_element: is_rest,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct JsFieldData {
    pub index: usize,
    pub docblock: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct JsMethodData {
    pub params: Vec<JsMethodParameter>,
    pub index: usize,
    pub docblock: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum JsMemberData {
    Method(JsMethodData),
    Field(JsFieldData),
}

#[derive(Serialize, Deserialize)]
pub struct JsReflectionData {
    pub fqcn: String,
    pub class_name: String,
    pub namespace: Option<String>,
    pub filename: Option<String>,
    pub members: Vec<JsMemberData>,
    pub docblock: Option<String>,
}

fn process_reflection_data(reflection_data: &ReflectionData) -> JsReflectionData {
    let class = &reflection_data.class;
    let namespace = reflection_data.namespace.clone();

    let members = class
        .body
        .iter()
        .enumerate()
        .filter_map(|(index, n)| match n {
            ClassMember::Constructor(c) => {
                let params = c
                    .params
                    .iter()
                    .enumerate()
                    .map(|(i, p)| match p {
                        ParamOrTsParamProp::TsParamProp(tp) => JsMethodParameter {
                            name: tp.param.as_ident().map(|i| i.sym.to_string()),
                            index: i,
                            has_default: tp.param.is_assign(),
                            scalar_default: tp
                                .param
                                .as_assign()
                                .and_then(|a| a.right.as_lit())
                                .and_then(|l| Scalar::try_from(l).ok()),
                            is_object_pattern: tp
                                .param
                                .as_assign()
                                .map(|a| a.left.is_object())
                                .unwrap_or(false),
                            is_array_pattern: tp
                                .param
                                .as_assign()
                                .map(|a| a.left.is_array())
                                .unwrap_or(false),
                            is_rest_element: false,
                        },
                        ParamOrTsParamProp::Param(p) => {
                            let mut p = JsMethodParameter::from(p);
                            p.index = i;

                            p
                        }
                    })
                    .collect();

                Some(JsMemberData::Method(JsMethodData {
                    params,
                    index,
                    docblock: reflection_data
                        .docblock
                        .get(&c.span)
                        .cloned()
                        .unwrap_or_default(),
                }))
            }
            ClassMember::Method(m) => {
                let params = m
                    .function
                    .params
                    .iter()
                    .enumerate()
                    .map(|(i, p)| {
                        let mut p = JsMethodParameter::from(p);
                        p.index = i;

                        p
                    })
                    .collect();

                Some(JsMemberData::Method(JsMethodData {
                    params,
                    index,
                    docblock: reflection_data
                        .docblock
                        .get(&m.span)
                        .cloned()
                        .unwrap_or_default(),
                }))
            }
            ClassMember::PrivateMethod(m) => {
                let params = m
                    .function
                    .params
                    .iter()
                    .enumerate()
                    .map(|(i, p)| {
                        let mut p = JsMethodParameter::from(p);
                        p.index = i;

                        p
                    })
                    .collect();

                Some(JsMemberData::Method(JsMethodData {
                    params,
                    index,
                    docblock: reflection_data
                        .docblock
                        .get(&m.span)
                        .cloned()
                        .unwrap_or_default(),
                }))
            }
            ClassMember::ClassProp(p) => Some(JsMemberData::Field(JsFieldData {
                index,
                docblock: reflection_data
                    .docblock
                    .get(&p.span)
                    .cloned()
                    .unwrap_or_default(),
            })),
            ClassMember::PrivateProp(p) => Some(JsMemberData::Field(JsFieldData {
                index,
                docblock: reflection_data
                    .docblock
                    .get(&p.span)
                    .cloned()
                    .unwrap_or_default(),
            })),
            ClassMember::AutoAccessor(a) => Some(JsMemberData::Field(JsFieldData {
                index,
                docblock: reflection_data
                    .docblock
                    .get(&a.span)
                    .cloned()
                    .unwrap_or_default(),
            })),
            _ => None,
        })
        .collect();

    let class_name = reflection_data.name.sym.to_string();
    let ns = namespace.as_deref();
    let fqcn = if ns.is_some_and(|n| !n.is_empty()) {
        format!("{}.{}", ns.unwrap(), class_name)
    } else {
        class_name.clone()
    };

    JsReflectionData {
        fqcn,
        class_name,
        namespace,
        filename: reflection_data.filename.clone(),
        members,
        docblock: reflection_data
            .docblock
            .get(&class.span)
            .cloned()
            .unwrap_or_default(),
    }
}

#[wasm_bindgen(js_name = getInternalReflectionData)]
pub fn get_js_reflection_data(class_id: &str) -> Result<JsValue, JsValue> {
    let Ok(class_id) = parse_uuid(class_id) else {
        return Ok(JsValue::undefined());
    };
    let Some(reflection_data) = get_reflection_data(&class_id) else {
        return Ok(JsValue::undefined());
    };

    Ok(serde_wasm_bindgen::to_value(&process_reflection_data(
        &reflection_data,
    ))?)
}

#[cfg(test)]
mod tests {
    use crate::parser::CodeParser;
    use crate::reflection::ReflectionData;
    use crate::wasm::reflection::{process_reflection_data, JsMemberData};
    use swc_common::DUMMY_SP;
    use swc_ecma_ast::Ident;

    #[test]
    pub fn should_process_method_parameters_correctly() -> anyhow::Result<()> {
        let code = r#"
/** class docblock */
export default class x {
    static #staticPrivateField;
    #privateField;
    accessor #privateAccessor;
    static staticPublicField;
    publicField;
    accessor publicAccessor;

    /** constructor docblock */
    constructor(@type(String) constructorParam1) {
    }

    /**
     * computed method docblock
     */
    [a()]() {}
    #privateMethod(a, b = 1, [c, d], {f, g}) {}

    /**
     * public method docblock
     */
    publicMethod({a, b} = {}, c = new Object(), ...x) {}
    static #staticPrivateMethod() {}
    static staticPublicMethod() {}

    get [a()]() {}
    set b(v) {}

    get #ap() {}
    set #bp(v) {}

    act(@type(String) param1) {}
    [a()](@type(String) param1) {}
    [Symbol.for('xtest')](@type(String) param1) {}
}

return x[Symbol.metadata].act[Symbol.parameters][0].type;
"#;

        let program = code.parse_program(None)?;
        let mut module = program.program.expect_module();
        let body = module.body.drain(..);
        let item = body.take(1).into_iter().nth(0).unwrap();
        let class_decl = item
            .expect_module_decl()
            .expect_export_default_decl()
            .decl
            .expect_class();

        let data = process_reflection_data(&ReflectionData {
            class: *class_decl.class,
            name: Ident {
                span: DUMMY_SP,
                sym: "x".into(),
                optional: false,
            },
            filename: None,
            namespace: None,
            docblock: Default::default(),
        });

        let JsMemberData::Method(method) = data.members.as_slice().iter().nth(8).unwrap() else {
            panic!("not a method");
        };
        assert!(method
            .params
            .as_slice()
            .iter()
            .nth(0)
            .unwrap()
            .name
            .is_some());
        assert!(method
            .params
            .as_slice()
            .iter()
            .nth(1)
            .unwrap()
            .name
            .is_some());

        Ok(())
    }
}
