# < definition scip-python python snapshot-util 0.1 vars_inside_scopes/__init__:

from typing import List

class X:
    items: List[int]

    def my_func(self):
        for x in self.items:
            y = x + 1

        if 5 in self.items:
            z = "oh ya"

