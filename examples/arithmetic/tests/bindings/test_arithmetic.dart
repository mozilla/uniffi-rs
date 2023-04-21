import "../arithmetic.dart";

import 'package:test/test.dart';

void main() {
  test('arithmetic works', () {
    final api = Api.load();

    // do {
    //     let _ = try add(a: 18446744073709551615, b: 1)
    //     fatalError("Should have thrown a IntegerOverflow exception!")
    // } catch ArithmeticError.IntegerOverflow {
    //     // It's okay!
    // }

    assert(api.add(2, 4) == 6, "add work");
    assert(api.add(4, 8) == 12, "add work");

    // do {
    //     let _ = try sub(0,1)
    //     fatalError("Should have thrown a IntegerOverflow exception!")
    // } catch ArithmeticError.IntegerOverflow {
    //     // It's okay!
    // }

    assert(api.sub(4, 2) == 2, "sub work");
    assert(api.sub(8, 4) == 4, "sub work");

    assert(api.div(8, 4) == 2, "div works");

    // We can't test panicking in Swift because we force unwrap the error in
    // `div`, which we can't catch.

    assert(api.equal(2, 2), "equal works");
    assert(api.equal(4, 4), "equal works");

    assert(!api.equal(2, 4), "non-equal works");
    assert(!api.equal(4, 8), "non-equal works");
  });
}
