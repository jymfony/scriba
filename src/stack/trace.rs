use super::{Frame, FILE_MAPPINGS};

/// Prepares stack trace using V8 stack trace API.
pub(crate) fn remap_stack_trace(
    error_message: &str,
    stack: &[Frame],
    previous: Option<String>,
) -> String {
    let mut processed = false;
    let mappings = FILE_MAPPINGS.read().unwrap();
    let new_stack = stack
        .iter()
        .filter_map(|frame| {
            if frame
                .function_name
                .as_deref()
                .is_some_and(|f| f == "_construct_jobject")
            {
                return None;
            }

            let file_name = frame.filename.as_deref().unwrap_or_default();
            if frame.is_native {
                return Some(frame.string_repr.clone());
            }

            let Some(source_map) = mappings.get(file_name) else {
                return Some(frame.string_repr.clone());
            };

            let line_no = frame.line_no - 1;
            let col_no = frame.col_no - 1;
            let Some(token) = source_map.0.lookup_token(line_no, col_no) else {
                return Some(frame.string_repr.clone());
            };

            let file_location = format!(
                "{}:{}:{}",
                file_name,
                token.get_src_line() + 1,
                token.get_src_col() + 1,
            );
            let mut function_name = frame.function_name.as_ref().cloned();
            if let Some(sv) = source_map.0.get_source_view(token.get_src_id()) {
                function_name = source_map
                    .0
                    .get_original_function_name(
                        line_no,
                        col_no,
                        function_name.unwrap_or_default().as_str(),
                        sv,
                    )
                    .map(|f| f.to_string());
            }

            if let Some(fun) = &function_name {
                if fun.starts_with("_anonymous_xΞ") {
                    function_name = None;
                }
            }

            let is_top_level = frame.is_top_level;
            let is_constructor = frame.is_constructor;
            let is_method_call = !(is_top_level || is_constructor);

            let generate_function_call = || {
                let mut call = "".to_string();

                if is_method_call {
                    if let Some(function_name) = &function_name {
                        if let Some(type_name) = &frame.type_name {
                            if !function_name.starts_with(type_name) {
                                call += type_name;
                                call += ".";
                            }
                        }

                        call += function_name;

                        if let Some(method_name) = &frame.method_name {
                            if !function_name.ends_with(method_name) {
                                call += " [as ";
                                call += method_name;
                                call += "]";
                            }
                        }
                    } else {
                        if let Some(type_name) = &frame.type_name {
                            call += type_name;
                            call += ".";
                        }

                        call += frame.method_name.as_deref().unwrap_or("<anonymous>");
                    }
                } else if is_constructor {
                    call += "new ";
                    call += &function_name.unwrap_or("<anonymous>".to_string());
                } else if let Some(function_name) = &function_name {
                    call += function_name;
                } else {
                    call += &file_location;
                    return call;
                }

                call += " (";
                call += &file_location;
                call += ")";

                call
            };

            processed = true;
            Some(format!(
                "{}{}{}",
                if frame.is_async { "async " } else { "" },
                if frame.is_promise_all {
                    format!("Promise.all (index {})", frame.promise_index)
                } else {
                    "".to_string()
                },
                generate_function_call()
            ))
        })
        .collect::<Vec<_>>();

    #[allow(clippy::unnecessary_unwrap)]
    if previous.is_some() && !processed {
        previous.unwrap()
    } else {
        String::from(error_message) + "\n\n    at " + new_stack.join("\n    at ").as_str()
    }
}
