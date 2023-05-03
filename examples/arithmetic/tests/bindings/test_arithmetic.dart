import "../arithmetic.dart";

import "package:test/test.dart";

void main() {
    test("arithmetic works", () {
        final api = Api.load();

        try {
            var _ = api.add(9223372036854775807, 1);
            throw Exception("Should have thrown a IntegerOverflow exception!");
        } on ArithmeticError catch (e) {
            // It's okay!
        } on Exception catch (e) {
            // It's okay!
        }

        assert(api.add(2, 4) == 6, "add work");
        assert(api.add(4, 8) == 12, "add work");

        try {
            var _ = api.sub(0, 1);
            // throw Exception("Should have thrown a IntegerOverflow exception!");
        } on ArithmeticError catch (e) {
            print("arithmetic error: $e");
            // It's okay!
        } on Exception catch (e) {
            // It's okay!
            print("general error: $e");
        }

        assert(api.sub(4, 2) == 2, "sub work");
        assert(api.sub(8, 4) == 4, "sub work");

        try {
            var _ = api.div(1, 0);
            // throw Exception("Should have thrown a DivideByZero exception!");
        } on ArithmeticError catch (e) {
            print("arithmetic error: $e");
            // It's okay!
        } on Exception catch (e) {
            // It's okay!
            print("general error: $e");
        }

        assert(api.div(8, 4) == 2, "div works");

        // We can't test panicking in Swift because we force unwrap the error in
        // `div`, which we can't catch.

        assert(api.equal(2, 2), "equal works");
        assert(api.equal(4, 4), "equal works");

        assert(!api.equal(2, 4), "non-equal works");
        assert(!api.equal(4, 8), "non-equal works");
    });
}
