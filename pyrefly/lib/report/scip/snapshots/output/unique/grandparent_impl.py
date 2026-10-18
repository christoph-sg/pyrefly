# < definition scip-python python snapshot-util 0.1 grandparent_impl/__init__:

class A:
    def grandparent(self) -> bool:
        return True

class B(A):
    ...

class C(B):
    def grandparent(self) -> bool:
        return False

