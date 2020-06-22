import uniffi.naming_conventions.*;

val camelCaseObject = camelCaseMethod(1)
val snakeCaseObject = snakeCaseMethod(2)
println("object1 is camelCase: ${camelCaseObject}")
println("object2 is not snake_case: ${snakeCaseObject}")
