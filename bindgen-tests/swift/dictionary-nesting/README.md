# Regression test for ambiguous nested dictionaries in Swift

Previously nested dictionary types, such as `[String: String]` and `[String: [String]]` would be exported under the same name (e.g. `FfiConverterDictionaryStringString`) leading to compilation issues.

This test ensures that nested dictionaries can be disambiguated in roughly the following way:

- `[String: String]` => `...DictionaryStringString`
- `[String: [String]]` => `...DictionaryStringSequenceString`
- `[String: [String: String]]` => `...DictionaryStringDictionaryStringString`
- `[String: [String: [String]]]` => `...DictionaryStringDictionaryStringSequenceString`
- etc
