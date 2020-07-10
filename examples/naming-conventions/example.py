from naming_conventions import *

camel_case_object = camel_case_method(1, Case.SNAKE_CASE)
snake_case_object = snake_case_method(2, Case.CAMEL_CASE)
print("object1 is camelCase: {}".format(camel_case_object))
print("object2 is snake_case: {}".format(snake_case_object))

case = get_camel_case()
print("case is UPPER_CAMEL_CASE?: {}".format(case))