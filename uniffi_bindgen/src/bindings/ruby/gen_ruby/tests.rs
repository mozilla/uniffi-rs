use super::{
    canonical_name, class_name_rb_inner, crate_name_from_module_path, filters, is_reserved_word,
    Config, RubyWrapper,
};
use crate::bindings::ruby::generate_ruby_bindings;
use crate::interface::{ComponentInterface, ObjectImpl, Type};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use uniffi_meta::NamespaceMetadata;

fn namespace(crate_name: &str, name: &str) -> NamespaceMetadata {
    NamespaceMetadata {
        crate_name: crate_name.to_string(),
        name: name.to_string(),
    }
}

fn ci_with_namespaces(
    udl: &str,
    crate_name: &str,
    namespaces: &[(&str, &str)],
) -> ComponentInterface {
    let mut ci = ComponentInterface::from_webidl(udl, crate_name).unwrap();
    let map = namespaces
        .iter()
        .map(|(c, n)| (c.to_string(), namespace(c, n)))
        .collect::<BTreeMap<_, _>>();
    ci.set_crate_to_namespace_map(map);
    ci
}

#[test]
fn when_reserved_word() {
    assert!(is_reserved_word("end"));
}

#[test]
fn when_not_reserved_word() {
    assert!(!is_reserved_word("ruby"));
}

#[test]
fn cdylib_name() {
    let config = Config::default();

    assert_eq!("uniffi", config.cdylib_name());

    let config = Config {
        cdylib_name: Some("todolist".to_string()),
        ..Default::default()
    };

    assert_eq!("todolist", config.cdylib_name());
}

#[test]
fn cdylib_path() {
    let config = Config::default();

    assert_eq!("", config.cdylib_path());
    assert!(!config.custom_cdylib_path());

    let config = Config {
        cdylib_path: Some("/foo/bar".to_string()),
        ..Default::default()
    };

    assert_eq!("/foo/bar", config.cdylib_path());
    assert!(config.custom_cdylib_path());
}

#[test]
fn module_name_falls_back_to_namespace_camel_case() {
    let config = Config::default();
    assert_eq!(config.module_name("foo_ns"), "FooNs");
}

#[test]
fn module_name_config_overrides_namespace() {
    let config = Config {
        module_name: Some("CustomFoo".into()),
        ..Default::default()
    };
    assert_eq!(config.module_name("foo_ns"), "CustomFoo");
}

#[test]
fn module_name_rejects_invalid_identifiers() {
    for name in ["foo", "Foo::Bar", "", "END", "1Foo"] {
        let config = Config {
            module_name: Some(name.into()),
            ..Default::default()
        };
        let err = config.validate_module_name().unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("module_name"), "name={name:?}: {msg}");
        assert!(msg.contains(name), "name={name:?}: {msg}");
    }
}

#[test]
fn module_name_accepts_valid_constant() {
    let config = Config {
        module_name: Some("UniffiOne".into()),
        ..Default::default()
    };
    config.validate_module_name().unwrap();
}

#[test]
fn generate_emits_configured_module_name() {
    let ci = ComponentInterface::from_webidl(
        r#"
        namespace foo_ns {};
        dictionary Foo { string value; };
        "#,
        "test",
    )
    .unwrap();
    let config = Config {
        module_name: Some("CustomFoo".into()),
        ..Default::default()
    };
    let src = generate_ruby_bindings(&config, &ci).unwrap();
    assert!(src.contains("module CustomFoo"), "{src}");
    assert!(src.contains("::CustomFoo."), "{src}");
    assert!(src.contains("::CustomFoo::RustBufferBuilderMixin"), "{src}");
    assert!(!src.contains("module FooNs"), "{src}");
    assert!(!src.contains("::FooNs."), "{src}");
}

#[test]
fn generate_rejects_invalid_module_name() {
    let ci = ComponentInterface::from_webidl("namespace test {};", "test").unwrap();
    let config = Config {
        module_name: Some("foo".into()),
        ..Default::default()
    };
    let err = generate_ruby_bindings(&config, &ci).unwrap_err();
    let msg = format!("{err:#}");
    assert!(msg.contains("module_name"), "{msg}");
}

#[test]
fn crate_name_from_module_path_normalizes_hyphens() {
    assert_eq!(crate_name_from_module_path("my-crate"), "my_crate");
    assert_eq!(crate_name_from_module_path("my_crate"), "my_crate");
    assert_eq!(crate_name_from_module_path("my-crate::sub"), "my_crate");
    assert_eq!(crate_name_from_module_path("my_crate::sub"), "my_crate");
}

#[test]
fn hyphenated_config_key_matches_underscored_module_path() {
    let mut config = Config::default();
    config
        .external_packages
        .insert("my-crate".into(), "Custom".into());
    config.normalize_external_package_keys().unwrap();
    assert_eq!(
        config.external_package_name("my_crate::sub", Some("my_ns")),
        "Custom"
    );
}

#[test]
fn underscored_config_key_matches_hyphenated_udl_module_path() {
    let mut config = Config::default();
    config
        .external_packages
        .insert("my_crate".into(), "Custom".into());
    config.normalize_external_package_keys().unwrap();
    assert_eq!(
        config.external_package_name("my-crate", Some("my_ns")),
        "Custom"
    );
}

#[test]
fn unmapped_crate_falls_back_to_namespace() {
    let config = Config::default();
    assert_eq!(
        config.external_package_name("other_crate", Some("other_ns")),
        "OtherNs"
    );
}

#[test]
fn normalize_external_package_keys_rejects_conflicting_values() {
    let mut config = Config::default();
    config
        .external_packages
        .insert("my-crate".into(), "A".into());
    config
        .external_packages
        .insert("my_crate".into(), "B".into());
    let err = config.normalize_external_package_keys().unwrap_err();
    assert!(err.to_string().contains("conflicting"));
}

#[test]
fn normalize_external_package_keys_allows_duplicate_equivalent_keys() {
    let mut config = Config::default();
    config
        .external_packages
        .insert("my-crate".into(), "Custom".into());
    config
        .external_packages
        .insert("my_crate".into(), "Custom".into());
    config.normalize_external_package_keys().unwrap();
    assert_eq!(config.external_packages.get("my_crate").unwrap(), "Custom");
    assert!(!config.external_packages.contains_key("my-crate"));
}

#[test]
fn is_external_module_treats_hyphenated_name_as_same_crate() {
    let ci = ComponentInterface::new("foo_bar");
    let wrapper = RubyWrapper::new(Config::default(), &ci);
    assert!(!wrapper.is_external_module("foo-bar"));
    assert!(!wrapper.is_external_module("foo_bar"));
    assert!(!wrapper.is_external_module("foo-bar::sub"));
    assert!(wrapper.is_external_module("other_crate"));
    assert!(wrapper.is_external_module("other-crate"));
}

const TWO_TYPES_UDL: &str = r#"
    namespace consumer {
        TypeA get_a();
        TypeB get_b();
    };

    [External="crate_a"]
    typedef dictionary TypeA;

    [External="crate_b"]
    typedef dictionary TypeB;
"#;

#[test]
fn external_mixin_modules_errors_when_external_crate_namespace_unresolved() {
    let ci = ComponentInterface::from_webidl(TWO_TYPES_UDL, "consumer").unwrap();
    let err = RubyWrapper::new(Config::default(), &ci)
        .external_mixin_modules()
        .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("crate_a") || msg.contains("crate_b"), "{msg}");
    assert!(
        msg.contains("Single-UDL generation is not supported"),
        "{msg}"
    );

    let render_err = generate_ruby_bindings(&Config::default(), &ci).unwrap_err();
    let render_msg = format!("{render_err:#}");
    assert!(
        render_msg.contains("Single-UDL generation is not supported"),
        "{render_msg}"
    );
}

#[test]
fn external_mixin_modules_collapses_two_types_from_same_crate() {
    let ci = ci_with_namespaces(
        r#"
        namespace consumer {
            TypeA get_a();
            TypeB get_b();
        };

        [External="crate_a"]
        typedef dictionary TypeA;

        [External="crate_a"]
        typedef dictionary TypeB;
        "#,
        "consumer",
        &[("consumer", "consumer"), ("crate_a", "ns_a")],
    );
    let mixins = RubyWrapper::new(Config::default(), &ci)
        .external_mixin_modules()
        .unwrap();
    assert_eq!(mixins.len(), 1);
    assert_eq!(mixins[0].require_path, "ns_a");
}

#[test]
fn external_mixin_modules_lists_each_crate() {
    let ci = ci_with_namespaces(
        TWO_TYPES_UDL,
        "consumer",
        &[
            ("consumer", "consumer"),
            ("crate_a", "ns_a"),
            ("crate_b", "ns_b"),
        ],
    );
    let mut mixins = RubyWrapper::new(Config::default(), &ci)
        .external_mixin_modules()
        .unwrap();
    mixins.sort_by(|a, b| a.require_path.cmp(&b.require_path));
    assert_eq!(mixins.len(), 2);
    assert_eq!(mixins[0].require_path, "ns_a");
    assert_eq!(mixins[1].require_path, "ns_b");
}

#[test]
fn external_mixin_modules_errors_on_camel_case_collision() {
    let ci = ci_with_namespaces(
        TWO_TYPES_UDL,
        "consumer",
        &[
            ("consumer", "consumer"),
            ("crate_a", "foo_bar"),
            ("crate_b", "fooBar"),
        ],
    );
    let err = RubyWrapper::new(Config::default(), &ci)
        .external_mixin_modules()
        .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("FooBar"), "{msg}");
    assert!(msg.contains("crate_a"), "{msg}");
    assert!(msg.contains("crate_b"), "{msg}");
}

#[test]
fn external_mixin_modules_errors_on_external_packages_collision() {
    let ci = ci_with_namespaces(
        TWO_TYPES_UDL,
        "consumer",
        &[
            ("consumer", "consumer"),
            ("crate_a", "ns_a"),
            ("crate_b", "ns_b"),
        ],
    );
    let mut config = Config::default();
    config
        .external_packages
        .insert("crate_a".into(), "Shared".into());
    config
        .external_packages
        .insert("crate_b".into(), "Shared".into());
    let err = RubyWrapper::new(config, &ci)
        .external_mixin_modules()
        .unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("Shared"), "{msg}");
    assert!(msg.contains("crate_a"), "{msg}");
    assert!(msg.contains("crate_b"), "{msg}");
}

#[test]
fn external_mixin_modules_ignores_unused_external_packages_entry() {
    let ci = ci_with_namespaces(
        r#"
        namespace consumer {
            TypeB get_b();
        };

        [External="crate_b"]
        typedef dictionary TypeB;
        "#,
        "consumer",
        &[
            ("consumer", "consumer"),
            ("crate_b", "ns_b"),
            ("crate_c", "ns_c"),
        ],
    );
    let mut config = Config::default();
    config
        .external_packages
        .insert("crate_b".into(), "NsB".into());
    config
        .external_packages
        .insert("crate_c".into(), "NsC".into());
    let mixins = RubyWrapper::new(config, &ci)
        .external_mixin_modules()
        .unwrap();
    assert_eq!(mixins.len(), 1);
    assert_eq!(mixins[0].require_path, "ns_b");
}

#[test]
fn external_mixin_modules_collapses_hyphenated_and_underscored_crate() {
    let ci = ci_with_namespaces(
        r#"
        namespace consumer {
            TypeA get_a();
            TypeB get_b();
        };

        [External="my-crate"]
        typedef dictionary TypeA;

        [External="my_crate"]
        typedef dictionary TypeB;
        "#,
        "consumer",
        &[("consumer", "consumer"), ("my_crate", "my_ns")],
    );
    let mixins = RubyWrapper::new(Config::default(), &ci)
        .external_mixin_modules()
        .unwrap();
    assert_eq!(mixins.len(), 1);
    assert_eq!(mixins[0].require_path, "my_ns");
}

#[test]
fn identity_custom_lower_coerces_integer_builtin() {
    let ci = ComponentInterface::from_webidl(
        r#"
        namespace test {
            Handle id(Handle h);
        };

        [Custom]
        typedef u64 Handle;
        "#,
        "test",
    )
    .unwrap();
    let src = generate_ruby_bindings(&Config::default(), &ci).unwrap();
    let lower = src
        .split("def self.uniffi_lower_TypeHandle(v)")
        .nth(1)
        .expect("identity lower for Handle");
    let body = lower.split("def self.").next().unwrap();
    assert!(
        body.contains("uniffi_in_range(v, \"u64\""),
        "identity u64 custom lower must coerce via uniffi_in_range, got:\n{body}"
    );
}

#[test]
fn identity_custom_lower_coerces_string_builtin() {
    let ci = ComponentInterface::from_webidl(
        r#"
        namespace test {
            Guid id(Guid g);
        };

        [Custom]
        typedef string Guid;
        "#,
        "test",
    )
    .unwrap();
    let src = generate_ruby_bindings(&Config::default(), &ci).unwrap();
    let lower = src
        .split("def self.uniffi_lower_TypeGuid(v)")
        .nth(1)
        .expect("identity lower for Guid");
    let body = lower.split("def self.").next().unwrap();
    assert!(
        body.contains("uniffi_utf8(v)"),
        "identity string custom lower must coerce via uniffi_utf8, got:\n{body}"
    );
}

#[test]
fn mixin_owner_module_roots_external_record() {
    let ci = ci_with_namespaces(
        TWO_TYPES_UDL,
        "consumer",
        &[
            ("consumer", "consumer"),
            ("crate_a", "ns_a"),
            ("crate_b", "ns_b"),
        ],
    );
    let w = RubyWrapper::new(Config::default(), &ci);
    let type_a = ci.get_type("TypeA").unwrap();
    assert_eq!(
        w.rust_buffer_write(&type_a).unwrap(),
        "::NsA::RustBufferBuilderMixin.write_TypeTypeA"
    );
    assert_eq!(
        w.rust_buffer_read(&type_a).unwrap(),
        "::NsA::RustBufferStreamMixin.read_TypeTypeA"
    );

    let local = Type::Record {
        module_path: "consumer".into(),
        name: "Local".into(),
    };
    assert_eq!(
        w.rust_buffer_write(&local).unwrap(),
        "::Consumer::RustBufferBuilderMixin.write_TypeLocal"
    );

    let optional_ext = Type::Optional {
        inner_type: Box::new(type_a.clone()),
    };
    assert_eq!(
        w.rust_buffer_write(&optional_ext).unwrap(),
        "::Consumer::RustBufferBuilderMixin.write_OptionalTypeTypeA"
    );

    assert_eq!(
        w.rust_buffer_write(&Type::String).unwrap(),
        "::Consumer::RustBufferBuilderMixin.write_string"
    );
}

#[test]
fn rust_buffer_write_qualifies_external_record() {
    let ci = ci_with_namespaces(
        TWO_TYPES_UDL,
        "consumer",
        &[
            ("consumer", "consumer"),
            ("crate_a", "ns_a"),
            ("crate_b", "ns_b"),
        ],
    );
    let w = RubyWrapper::new(Config::default(), &ci);
    let type_a = ci.get_type("TypeA").unwrap();
    let callee = w.rust_buffer_write(&type_a).unwrap();
    assert_eq!(callee, "::NsA::RustBufferBuilderMixin.write_TypeTypeA");
    assert!(
        !callee.contains('('),
        "callee must not bake argument list: {callee}"
    );
}

#[test]
fn generated_consumer_does_not_include_foreign_mixins() {
    let ci = ci_with_namespaces(
        TWO_TYPES_UDL,
        "consumer",
        &[
            ("consumer", "consumer"),
            ("crate_a", "ns_a"),
            ("crate_b", "ns_b"),
        ],
    );
    let src = generate_ruby_bindings(&Config::default(), &ci).unwrap();
    assert!(src.contains("require 'ns_a'"), "{src}");
    assert!(src.contains("require 'ns_b'"), "{src}");
    assert!(
        src.contains("::NsA::RustBufferBuilderMixin.write_TypeTypeA"),
        "{src}"
    );
    assert!(
        !src.contains("include ::NsA::RustBufferBuilderMixin"),
        "{src}"
    );
    assert!(
        !src.contains("include ::NsB::RustBufferBuilderMixin"),
        "{src}"
    );
}

#[test]
fn generated_local_only_uses_module_functions() {
    let ci = ComponentInterface::from_webidl(
        r#"
        namespace test {};
        dictionary Foo { string value; };
        "#,
        "test",
    )
    .unwrap();
    let src = generate_ruby_bindings(&Config::default(), &ci).unwrap();
    assert!(src.contains("def self.write_TypeFoo(builder, v)"), "{src}");
    assert!(
        src.contains("::Test::RustBufferBuilderMixin.write_TypeFoo"),
        "{src}"
    );
    assert!(!src.contains("builder.write_TypeFoo"), "{src}");
    assert!(!src.contains("include RustBufferBuilderMixin"), "{src}");
}

#[test]
fn error_reader_is_method_object() {
    let ci = ComponentInterface::from_webidl(
        r#"
        namespace test {
            [Throws=Boom]
            u32 go();
        };
        [Error]
        enum Boom { "Oops" };
        "#,
        "test",
    )
    .unwrap();
    let src = generate_ruby_bindings(&Config::default(), &ci).unwrap();
    assert!(
        src.contains("rust_call_with_error(::Test::RustBufferStreamMixin.method(:read_TypeBoom)"),
        "{src}"
    );
}

#[test]
fn templates_have_no_receiver_mixin_calls() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/bindings/ruby/templates");
    for entry in fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|s| s.to_str()) != Some("rb") {
            continue;
        }
        let src = fs::read_to_string(&path).unwrap();
        let name = path.file_name().unwrap().to_string_lossy();
        for (i, line) in src.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                continue;
            }
            let loc = format!("{name}:{}", i + 1);
            if line.contains("self.write_{{") {
                assert!(
                    line.contains("def self.write_{{"),
                    "{loc}: leftover self.write_ {line}"
                );
            }
            if line.contains("self.read_{{") {
                assert!(
                    line.contains("def self.read_{{"),
                    "{loc}: leftover self.read_ {line}"
                );
            }
            assert!(
                !line.contains("builder.write_{{") && !line.contains("stream.read_{{"),
                "{loc}: leftover facade {line}"
            );
            if line.contains("write_{{") {
                assert!(
                    line.contains("def self.write_{{"),
                    "{loc}: write_{{{{ not a module-function def: {line}"
                );
            }
            if line.contains("read_{{") {
                assert!(
                    line.contains("def self.read_{{"),
                    "{loc}: read_{{{{ not a module-function def: {line}"
                );
            }
        }
        if name == "RustBufferBuilder.rb" {
            for (i, line) in src.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with('#') || trimmed.starts_with("def pack_into") {
                    continue;
                }
                if trimmed.contains("pack_into") {
                    assert!(
                        line.contains("builder.pack_into"),
                        "RustBufferBuilder.rb:{}: pack_into without builder. receiver: {line}",
                        i + 1
                    );
                }
            }
        }
        if name == "RustBufferStream.rb" {
            for (i, line) in src.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with('#') || trimmed.starts_with("def unpack_from") {
                    continue;
                }
                if trimmed.contains("unpack_from") {
                    assert!(
                        line.contains("stream.unpack_from"),
                        "RustBufferStream.rb:{}: unpack_from without stream. receiver: {line}",
                        i + 1
                    );
                }
            }
        }
    }
}

#[test]
fn qualify_roots_external_module() {
    assert_eq!(filters::qualify("Foo", Some("Mod")), "::Mod::Foo");
    assert_eq!(filters::qualify("", Some("Mod")), "::Mod::");
    assert_eq!(filters::qualify("Foo", None), "Foo");
}

#[test]
fn canonical_names() {
    assert_eq!(canonical_name(&Type::UInt8), "u8");
    assert_eq!(canonical_name(&Type::String), "string");
    assert_eq!(canonical_name(&Type::Bytes), "bytes");
    assert_eq!(
        canonical_name(&Type::Optional {
            inner_type: Box::new(Type::Sequence {
                inner_type: Box::new(Type::Object {
                    module_path: "anything".to_string(),
                    name: "Example".into(),
                    imp: ObjectImpl::Struct,
                })
            })
        }),
        "OptionalSequenceTypeExample"
    );

    let map = Type::Map {
        key_type: Box::new(Type::UInt32),
        value_type: Box::new(Type::UInt32),
    };
    assert_eq!(canonical_name(&map), "MapU32U32");
    assert_eq!(
        canonical_name(&Type::Enum {
            module_path: "foo".to_string(),
            name: "HTMLError".to_string()
        }),
        "TypeHTMLError"
    );
}

#[test]
fn class_name() {
    assert_eq!(class_name_rb_inner("Example"), "Example");
}

#[test]
fn enum_lift_consume_into_matches_method_name() {
    // heck would turn TypeHTMLError into TypeHtmlError; the generated
    // consume_into_* method uses canonical_name, so the lift call site must too.
    let ci = ComponentInterface::from_webidl(
        r#"
        namespace test {
            HTMLError id();
        };
        enum HTMLError { "InvalidHTML" };
        "#,
        "test",
    )
    .unwrap();
    let src = generate_ruby_bindings(&Config::default(), &ci).unwrap();
    assert!(
        src.contains("def consume_into_TypeHTMLError"),
        "method def:\n{src}"
    );
    assert!(
        src.contains("result.consume_into_TypeHTMLError"),
        "lift call site must match the method, not heck(canonical_name):\n{src}"
    );
    assert!(
        !src.contains("consume_into_TypeHtmlError"),
        "heck-rewritten consume_into must not appear:\n{src}"
    );
}
