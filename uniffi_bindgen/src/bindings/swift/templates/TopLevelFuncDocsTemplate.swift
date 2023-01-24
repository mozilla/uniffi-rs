
/// fun name: {{func.name()}}
/// for help: https://developer.apple.com/documentation/docc
/// - Parameters:
{% for arg in func.arguments() -% }
///   - {{ arg.name() }}: argument description
{% endfor % }
/// - Returns: The sloth's energy level after eating.
