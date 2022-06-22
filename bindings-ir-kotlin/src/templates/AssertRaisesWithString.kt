try {
    {{ stmt }}
    throw AssertionError("exception not thrown")
} catch (e: Throwable) {
    assert(e.message == {{ string_value }})
}
