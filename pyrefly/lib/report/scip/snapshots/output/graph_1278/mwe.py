# < definition scip-python python snapshot-util 0.1 mwe/__init__:

from typing import TypeVar, Generic, Callable, Iterator, ParamSpec

_T_co = TypeVar("_T_co")
#^^^^ definition  snapshot-util 0.1 mwe/_T_co.
_P = ParamSpec("_P")
#^ definition  snapshot-util 0.1 mwe/_P.

class X(Generic[_T_co]):
#     ^ definition  snapshot-util 0.1 mwe/X#
    pass

def decorate(func: Callable[_P, Iterator[_T_co]]) -> Callable[_P, X[_T_co]]: ...
#   ^^^^^^^^ definition  snapshot-util 0.1 mwe/decorate().
#            ^^^^ definition  snapshot-util 0.1 mwe/decorate().(func)

class Foo:
#     ^^^ definition  snapshot-util 0.1 mwe/Foo#
    @decorate
    def foo(self) -> Iterator[None]: ...
#       ^^^ definition  snapshot-util 0.1 mwe/Foo#foo().
#           ^^^^ definition  snapshot-util 0.1 mwe/Foo#foo().(self)

@decorate
def noop():
#   ^^^^ definition  snapshot-util 0.1 mwe/noop().
    yield

class FooImpl(Foo):
#     ^^^^^^^ definition  snapshot-util 0.1 mwe/FooImpl#
    def foo(self):
#       ^^^ definition  snapshot-util 0.1 mwe/FooImpl#foo().
#           ^^^^ definition  snapshot-util 0.1 mwe/FooImpl#foo().(self)
        return noop()
