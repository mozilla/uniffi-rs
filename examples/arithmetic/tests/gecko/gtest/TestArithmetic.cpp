/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

#include "gtest/gtest.h"

#include "mozilla/Arithmetic.h"

using namespace mozilla;

TEST(TestArithmetic, Add)
{
  ASSERT_EQ(arithmetic::Add(2, 4).unwrap(), 6u);
  ASSERT_EQ(arithmetic::Add(4, 8).unwrap(), 12u);

  auto result = arithmetic::Add(18446744073709551615ull, 1);
  ASSERT_EQ(result.inspectErr().Type(),
            arithmetic::ArithmeticError::IntegerOverflow);
  nsAutoCString message;
  result.inspectErr().Message(message);
  ASSERT_TRUE(StringBeginsWith(message, "Integer overflow on an operation"_ns));
}
