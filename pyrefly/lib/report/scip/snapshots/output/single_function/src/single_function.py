# < definition scip-python python snapshot-util 0.1 `src.single_function`/__init__:

def my_cool_function(a: str) -> str:
    x = ", world"
    return a + x

def my_cool_function_2(a: str):
    x = ", world"
    return (lambda y: a + x + y)("oh no")

def next_level():
    return my_cool_function

my_cool_function("hello")
next_level()(a = "wow")

