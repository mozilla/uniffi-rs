import uniffi.arithmetic.*;
println("Alrighty, let's do some arithmetic IN RUST!")
println("2 + 3 = ${add(2, 3, Overflow.SATURATING)}")
