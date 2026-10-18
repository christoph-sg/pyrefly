# < definition scip-python python snapshot-util 0.1 property_access/__init__:

from typing import Sequence

class PropertyClass:
    def __init__(self):
        pass

    @property
    def prop_ref(self):
        return 5


xs = [PropertyClass()]

def usage(xs: Sequence[PropertyClass]):
    def nested():
        for x in xs:
            print(x.prop_ref)

