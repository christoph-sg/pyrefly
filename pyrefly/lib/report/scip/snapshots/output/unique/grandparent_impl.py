# < definition scip-python python snapshot-util 0.1 grandparent_impl/__init__:

class A:
#     ^ definition  snapshot-util 0.1 grandparent_impl/A#
    def grandparent(self) -> bool:
#       ^^^^^^^^^^^ definition  snapshot-util 0.1 grandparent_impl/A#grandparent().
#                   ^^^^ definition  snapshot-util 0.1 grandparent_impl/A#grandparent().(self)
        return True

class B(A):
#     ^ definition  snapshot-util 0.1 grandparent_impl/B#
    ...

class C(B):
#     ^ definition  snapshot-util 0.1 grandparent_impl/C#
    def grandparent(self) -> bool:
#       ^^^^^^^^^^^ definition  snapshot-util 0.1 grandparent_impl/C#grandparent().
#                   ^^^^ definition  snapshot-util 0.1 grandparent_impl/C#grandparent().(self)
        return False

