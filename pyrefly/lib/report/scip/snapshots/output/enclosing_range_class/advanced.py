# < definition scip-python python snapshot-util 0.1 advanced/__init__:

# format-options: showRanges
def class_decorator(cls):
#   ^^^^^^^^^^^^^^^ definition  snapshot-util 0.1 advanced/class_decorator().
#                   ^^^ definition  snapshot-util 0.1 advanced/class_decorator().(cls)
    def wrapper(*args, **kwargs):
#       ^^^^^^^ definition  snapshot-util 0.1 advanced/class_decorator().wrapper().
#                ^^^^ definition  snapshot-util 0.1 advanced/class_decorator().wrapper().(args)
#                        ^^^^^^ definition  snapshot-util 0.1 advanced/class_decorator().wrapper().(kwargs)
        return cls(*args, **kwargs)
#              ^^^ reference  snapshot-util 0.1 advanced/class_decorator().(cls)
#                   ^^^^ reference  snapshot-util 0.1 advanced/class_decorator().wrapper().(args)
#                           ^^^^^^ reference  snapshot-util 0.1 advanced/class_decorator().wrapper().(kwargs)
    return wrapper
#          ^^^^^^^ reference  snapshot-util 0.1 advanced/class_decorator().wrapper().

@class_decorator
#^^^^^^^^^^^^^^^ reference  snapshot-util 0.1 advanced/class_decorator().
class Test:
#     ^^^^ definition  snapshot-util 0.1 advanced/Test#
    def __init__(self, x: float):
#       ^^^^^^^^ definition  snapshot-util 0.1 advanced/Test#__init__().
#                ^^^^ definition  snapshot-util 0.1 advanced/Test#__init__().(self)
#                      ^ definition  snapshot-util 0.1 advanced/Test#__init__().(x)
#                         ^^^^^ reference  python-stdlib 3.13 builtins/float#
        self.x = x
#       ^^^^ reference  snapshot-util 0.1 advanced/Test#__init__().(self)
#            ^ definition  snapshot-util 0.1 advanced/Test#x.
#                ^ reference  snapshot-util 0.1 advanced/Test#__init__().(x)

    def test(self) -> float:
#       ^^^^ definition  snapshot-util 0.1 advanced/Test#test().
#            ^^^^ definition  snapshot-util 0.1 advanced/Test#test().(self)
#                     ^^^^^ reference  python-stdlib 3.13 builtins/float#
        return self.x
#              ^^^^ reference  snapshot-util 0.1 advanced/Test#test().(self)

