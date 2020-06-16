import uniffi.arithmetic.*;
println("Alrighty, let's do some arithmetic FROM RUST, IN KOTLIN!")
println("2 + 3 = ${add(2, 3, Overflow.SATURATING)}")
