#[cfg(not(test))]
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use swc_common::Span;
use swc_ecma_ast::{Class, Ident};
use uuid::Uuid;

#[cfg(test)]
thread_local! {
    static CLASS_REGISTRY: RwLock<HashMap<Uuid, Arc<ReflectionData>>> = RwLock::new(Default::default());
}

#[cfg(not(test))]
lazy_static! {
    static ref CLASS_REGISTRY: RwLock<HashMap<Uuid, Arc<ReflectionData>>> =
        RwLock::new(Default::default());
}

pub struct ReflectionData {
    pub class: Class,
    pub name: Ident,
    pub filename: Option<String>,
    pub namespace: Option<String>,
    pub docblock: HashMap<Span, Option<String>>,
}

impl ReflectionData {
    pub fn new(
        class: &Class,
        name: Ident,
        filename: Option<&str>,
        namespace: Option<&str>,
        docblock: HashMap<Span, Option<String>>,
    ) -> Self {
        Self {
            class: class.clone(),
            name,
            filename: filename.map(|s| s.to_string()),
            namespace: namespace.map(|s| s.to_string()),
            docblock,
        }
    }
}

pub(crate) fn register_class(class_id: &Uuid, data: ReflectionData) {
    #[cfg(not(test))]
    {
        let mut registry = CLASS_REGISTRY.write().unwrap();

        debug_assert!(registry.get(class_id).is_none());
        registry.insert(class_id.clone(), Arc::new(data));
    }

    #[cfg(test)]
    {
        CLASS_REGISTRY.with(|lock| {
            let mut registry = lock.write().unwrap();

            debug_assert!(registry.get(class_id).is_none());
            registry.insert(class_id.clone(), Arc::new(data));
        });
    }
}

pub(crate) fn get_reflection_data<'a>(class_id: &Uuid) -> Option<Arc<ReflectionData>> {
    #[cfg(not(test))]
    let data = {
        let registry = CLASS_REGISTRY.read().unwrap();
        registry.get(class_id).map(Clone::clone)
    };

    #[cfg(test)]
    let data = {
        CLASS_REGISTRY.with(|lock| {
            let registry = lock.read().unwrap();
            registry.get(class_id).map(Clone::clone)
        })
    };

    data
}
