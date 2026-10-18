# < definition scip-python python snapshot-util 0.1 class_nohint/__init__:

# format-options: showDocs

class Example:
#     ^^^^^^^ definition  snapshot-util 0.1 class_nohint/Example#
    # Note, only y has a type hint
    y: int
#   ^ definition  snapshot-util 0.1 class_nohint/Example#y.

    def __init__(self, in_val):
#       ^^^^^^^^ definition  snapshot-util 0.1 class_nohint/Example#__init__().
#                ^^^^ definition  snapshot-util 0.1 class_nohint/Example#__init__().(self)
#                      ^^^^^^ definition  snapshot-util 0.1 class_nohint/Example#__init__().(in_val)
        self.x = in_val
#            ^ definition  snapshot-util 0.1 class_nohint/Example#x.
        self.x = self.x + 1
        self.y = in_val

    def something(self):
#       ^^^^^^^^^ definition  snapshot-util 0.1 class_nohint/Example#something().
#                 ^^^^ definition  snapshot-util 0.1 class_nohint/Example#something().(self)
        print(self.x)
        print(self.y)

