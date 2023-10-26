use crate::parse_uuid;
use crate::reflection::get_reflection_data;
use serde::{Deserialize, Serialize};
use swc_ecma_ast::*;
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsMethodParameter {
    pub name: Option<String>,
    pub index: usize,
    pub has_default: bool,
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

        JsMethodParameter {
            name: pat.as_ident().map(|i| i.sym.to_string()),
            index: 0,
            has_default: pat.is_assign(),
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

#[wasm_bindgen(js_name = getInternalReflectionData)]
pub fn get_js_reflection_data(class_id: &str) -> Result<JsValue, JsValue> {
    let Ok(class_id) = parse_uuid(class_id) else {
        return Ok(JsValue::undefined());
    };
    let Some(reflection_data) = get_reflection_data(&class_id) else {
        return Ok(JsValue::undefined());
    };

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
    let fqcn = if let Some(ns) = namespace.as_deref() {
        format!("{}.{}", ns, class_name)
    } else {
        class_name.clone()
    };

    Ok(serde_wasm_bindgen::to_value(&JsReflectionData {
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
    })?)
}
