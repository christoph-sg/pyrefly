# < definition scip-python python snapshot-util 0.1 inherits_class/__init__:

class A:
#     ^ definition  snapshot-util 0.1 inherits_class/A#
    def x(self) -> int:
#       ^ definition  snapshot-util 0.1 inherits_class/A#x().
#         ^^^^ definition  snapshot-util 0.1 inherits_class/A#x().(self)
        raise NotImplemented

    def matched_despite_different_type(self, x: int):
#       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ definition  snapshot-util 0.1 inherits_class/A#matched_despite_different_type().
#                                      ^^^^ definition  snapshot-util 0.1 inherits_class/A#matched_despite_different_type().(self)
#                                            ^ definition  snapshot-util 0.1 inherits_class/A#matched_despite_different_type().(x)
        pass

class B(A):
#     ^ definition  snapshot-util 0.1 inherits_class/B#
    def x(self) -> int:
#       ^ definition  snapshot-util 0.1 inherits_class/B#x().
#         ^^^^ definition  snapshot-util 0.1 inherits_class/B#x().(self)
        return 5

    def matched_despite_different_type(self, x: int, y: int):
#       ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ definition  snapshot-util 0.1 inherits_class/B#matched_despite_different_type().
#                                      ^^^^ definition  snapshot-util 0.1 inherits_class/B#matched_despite_different_type().(self)
#                                            ^ definition  snapshot-util 0.1 inherits_class/B#matched_despite_different_type().(x)
#                                                    ^ definition  snapshot-util 0.1 inherits_class/B#matched_despite_different_type().(y)
        pass

    def unrelated(self):
#       ^^^^^^^^^ definition  snapshot-util 0.1 inherits_class/B#unrelated().
#                 ^^^^ definition  snapshot-util 0.1 inherits_class/B#unrelated().(self)
        pass

