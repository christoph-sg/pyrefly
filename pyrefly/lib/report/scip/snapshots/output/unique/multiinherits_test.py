# < definition scip-python python snapshot-util 0.1 multiinherits_test/__init__:

class Left:
#     ^^^^ definition  snapshot-util 0.1 multiinherits_test/Left#
    def one(self) -> int:
#       ^^^ definition  snapshot-util 0.1 multiinherits_test/Left#one().
#           ^^^^ definition  snapshot-util 0.1 multiinherits_test/Left#one().(self)
        return 1

    def shared(self) -> bool:
#       ^^^^^^ definition  snapshot-util 0.1 multiinherits_test/Left#shared().
#              ^^^^ definition  snapshot-util 0.1 multiinherits_test/Left#shared().(self)
        return False

class Right:
#     ^^^^^ definition  snapshot-util 0.1 multiinherits_test/Right#
    def two(self):
#       ^^^ definition  snapshot-util 0.1 multiinherits_test/Right#two().
#           ^^^^ definition  snapshot-util 0.1 multiinherits_test/Right#two().(self)
        return 2

    def shared(self) -> bool:
#       ^^^^^^ definition  snapshot-util 0.1 multiinherits_test/Right#shared().
#              ^^^^ definition  snapshot-util 0.1 multiinherits_test/Right#shared().(self)
        return False

class Multi(Left, Right):
#     ^^^^^ definition  snapshot-util 0.1 multiinherits_test/Multi#
    def one(self) -> int:
#       ^^^ definition  snapshot-util 0.1 multiinherits_test/Multi#one().
#           ^^^^ definition  snapshot-util 0.1 multiinherits_test/Multi#one().(self)
        return 1

    def two(self):
#       ^^^ definition  snapshot-util 0.1 multiinherits_test/Multi#two().
#           ^^^^ definition  snapshot-util 0.1 multiinherits_test/Multi#two().(self)
        return 2

    def three(self):
#       ^^^^^ definition  snapshot-util 0.1 multiinherits_test/Multi#three().
#             ^^^^ definition  snapshot-util 0.1 multiinherits_test/Multi#three().(self)
        return 3

    def shared(self) -> bool:
#       ^^^^^^ definition  snapshot-util 0.1 multiinherits_test/Multi#shared().
#              ^^^^ definition  snapshot-util 0.1 multiinherits_test/Multi#shared().(self)
        return True

