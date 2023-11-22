# Docstrings

UDL file supports docstring comments. The comments are emitted in generated bindings without any
transformations. What you see in UDL is what you get in generated bindings. The only change made to
UDL comments are the comment syntax specific to each language. Docstrings can be used for most
declarations in UDL file. Docstrings are parsed as AST nodes, so incorrectly placed docstrings will
generate parse errors. Docstrings in UDL are comments prefixed with `///`.

## Docstrings in UDL
```java
/// The list of supported capitalization options
enum Capitalization {
    /// Lowercase, i.e. `hello, world!`
    Lower,

    /// Uppercase, i.e. `Hello, World!`
    Upper
};

namespace example {
    /// Return a greeting message, using `capitalization` for capitalization
    string hello_world(Capitalization capitalization);
}
```

## Docstrings in generated Kotlin bindings
```kotlin
/**
 * The list of supported capitalization options
 */
enum class Capitalization {
    /**
     * Lowercase, i.e. `hello, world!`
     */
    LOWER,

    /**
     * Uppercase, i.e. `Hello, World!`
     */
    UPPER;
}

/**
 * Return a greeting message, using `capitalization` for capitalization
 */
fun `helloWorld`(`capitalization`: Capitalization): String { .. }
```

## Docstrings in generated Swift bindings
```swift
/**
 * The list of supported capitalization options
 */
public enum Capitalization {
    /**
     * Lowercase, i.e. `hello, world!`
     */
    case lower

    /**
     * Uppercase, i.e. `Hello, World!`
     */
    case upper
}

/**
 * Return a greeting message, using `capitalization` for capitalization
 */
public func helloWorld(capitalization: Capitalization) -> String;
```

## Docstrings in generated Python bindings
```python
class Capitalization(enum.Enum):
    """The list of supported capitalization options"""

    LOWER = 1
    """Lowercase, i.e. `hello, world!`"""

    UPPER = 2
    """Uppercase, i.e. `Hello, World!`"""

def hello_world(capitalization: "Capitalization") -> "str":
    """Return a greeting message, using `capitalization` for capitalization"""
    ..
```
