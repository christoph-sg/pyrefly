# < definition scip-python python snapshot-util 0.1 vars_inside_scopes/__init__:

from typing import List

class X:
#     ^ definition  snapshot-util 0.1 vars_inside_scopes/X#
    items: List[int]
#   ^^^^^ definition  snapshot-util 0.1 vars_inside_scopes/X#items.

    def my_func(self):
#       ^^^^^^^ definition  snapshot-util 0.1 vars_inside_scopes/X#my_func().
#               ^^^^ definition  snapshot-util 0.1 vars_inside_scopes/X#my_func().(self)
        for x in self.items:
#           ^ definition local 0
            y = x + 1
#           ^ definition local 1

        if 5 in self.items:
            z = "oh ya"
#           ^ definition local 2

