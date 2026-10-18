# < definition scip-python python snapshot-util 0.1 `src.single_class`/__init__:

class ExampleClass:
#     ^^^^^^^^^^^^ definition  snapshot-util 0.1 `src.single_class`/ExampleClass#
    a: int
#   ^ definition  snapshot-util 0.1 `src.single_class`/ExampleClass#a.
    b: int
#   ^ definition  snapshot-util 0.1 `src.single_class`/ExampleClass#b.
    c: str
#   ^ definition  snapshot-util 0.1 `src.single_class`/ExampleClass#c.

    static_var = "Hello World"
#   ^^^^^^^^^^ definition  snapshot-util 0.1 `src.single_class`/ExampleClass#static_var.

    def __init__(self, a: int, b: int):
#       ^^^^^^^^ definition  snapshot-util 0.1 `src.single_class`/ExampleClass#__init__().
#                ^^^^ definition  snapshot-util 0.1 `src.single_class`/ExampleClass#__init__().(self)
#                      ^ definition  snapshot-util 0.1 `src.single_class`/ExampleClass#__init__().(a)
#                              ^ definition  snapshot-util 0.1 `src.single_class`/ExampleClass#__init__().(b)
        local_c = ", world!"
#       ^^^^^^^ definition local 0

        self.a = a
        self.b = b
        self.c = "hello" + local_c

