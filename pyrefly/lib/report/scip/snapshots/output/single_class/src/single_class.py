# < definition scip-python python snapshot-util 0.1 `src.single_class`/__init__:

class ExampleClass:
    a: int
    b: int
    c: str

    static_var = "Hello World"

    def __init__(self, a: int, b: int):
        local_c = ", world!"

        self.a = a
        self.b = b
        self.c = "hello" + local_c

