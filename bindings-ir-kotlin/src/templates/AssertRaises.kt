try {
    {{ stmt }}
    throw AssertionError("{{ name }} not thrown")
} catch (e: {{ name }} ) {
    // pass
}
