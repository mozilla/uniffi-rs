import uniffi.naming_conventions.*;

val camelCaseObject = camelCaseMethod(1, Case.CAMEL_CASE)
val snakeCaseObject = snakeCaseMethod(2, Case.SNAKE_CASE)

println("object1 is camelCase: ${camelCaseObject}")
println("object2 is not snake_case: ${snakeCaseObject}")

val case = getSnakeCase()
println("case is SNAKE_CASE?: ${case}")