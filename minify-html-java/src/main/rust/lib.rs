use jni::objects::JClass;
use jni::objects::JObject;
use jni::objects::JString;
use jni::sys::jstring;
use jni::JNIEnv;
use minify_html::minify as minify_html_native;
use minify_html::Cfg;
use std::ptr;
use std::str::from_utf8;

fn throw_minify_exception(env: &JNIEnv, message: &str) {
    let exception_class = "in/wilsonl/minifyhtml/MinifyHtmlException";
    let _ = env.throw_new(exception_class, message);
}

fn build_cfg(env: &JNIEnv, obj: &JObject) -> Result<Cfg, String> {
    macro_rules! get_boolean_field {
        ($method:expr) => {
            env.call_method(*obj, $method, "()Z", &[])
                .map_err(|e| format!("Failed to call method {}: {}", $method, e))?
                .z()
                .map_err(|e| format!("Failed to get boolean value from {}: {}", $method, e))?
        };
    }

    #[rustfmt::skip]
    let cfg = Cfg {
        // BEGIN CONFIGURATION FIELDS
        allow_noncompliant_unquoted_attribute_values: get_boolean_field!("isAllowNoncompliantUnquotedAttributeValues"),
        allow_optimal_entities: get_boolean_field!("isAllowOptimalEntities"),
        allow_removing_spaces_between_attributes: get_boolean_field!("isAllowRemovingSpacesBetweenAttributes"),
        keep_closing_tags: get_boolean_field!("isKeepClosingTags"),
        keep_comments: get_boolean_field!("isKeepComments"),
        keep_html_and_head_opening_tags: get_boolean_field!("isKeepHtmlAndHeadOpeningTags"),
        keep_input_type_text_attr: get_boolean_field!("isKeepInputTypeTextAttr"),
        keep_ssi_comments: get_boolean_field!("isKeepSsiComments"),
        minify_css: get_boolean_field!("isMinifyCss"),
        minify_doctype: get_boolean_field!("isMinifyDoctype"),
        minify_js: get_boolean_field!("isMinifyJs"),
        preserve_brace_template_syntax: get_boolean_field!("isPreserveBraceTemplateSyntax"),
        preserve_chevron_percent_template_syntax: get_boolean_field!("isPreserveChevronPercentTemplateSyntax"),
        remove_bangs: get_boolean_field!("isRemoveBangs"),
        remove_processing_instructions: get_boolean_field!("isRemoveProcessingInstructions"),
        // END CONFIGURATION FIELDS
    };

    Ok(cfg)
}

#[no_mangle]
pub extern "system" fn Java_in_wilsonl_minifyhtml_MinifyHtml_minifyRs(
    env: JNIEnv,
    _class: JClass,
    input: JString,
    cfg: JObject,
) -> jstring {
    if input.is_null() {
        throw_minify_exception(&env, "Input HTML string cannot be null");
        return ptr::null_mut();
    }

    if cfg.is_null() {
        throw_minify_exception(&env, "Configuration object cannot be null");
        return ptr::null_mut();
    }

    let source = match env.get_string(input) {
        Ok(java_str) => {
            match String::from(java_str) {
                s => s
            }
        },
        Err(e) => {
            throw_minify_exception(&env, &format!("Failed to get input string: {}", e));
            return ptr::null_mut();
        }
    };

    let config = match build_cfg(&env, &cfg) {
        Ok(cfg) => cfg,
        Err(e) => {
            throw_minify_exception(&env, &format!("Failed to build configuration: {}", e));
            return ptr::null_mut();
        }
    };
    let code = source.into_bytes();
    let out_code = match std::panic::catch_unwind(|| {
        minify_html_native(&code, &config)
    }) {
        Ok(result) => result,
        Err(_) => {
            throw_minify_exception(&env, "HTML minification failed due to internal error");
            return ptr::null_mut();
        }
    };
    let out_code_str = match from_utf8(&out_code) {
        Ok(s) => s,
        Err(e) => {
            throw_minify_exception(&env, &format!("Failed to convert minified output to UTF-8: {}", e));
            return ptr::null_mut();
        }
    };

    match env.new_string(out_code_str) {
        Ok(java_string) => java_string.into_inner(),
        Err(e) => {
            throw_minify_exception(&env, &format!("Failed to create output Java string: {}", e));
            ptr::null_mut()
        }
    }
}
