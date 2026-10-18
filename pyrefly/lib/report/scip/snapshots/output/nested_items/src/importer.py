# < definition scip-python python snapshot-util 0.1 `src.importer`/__init__:

from foo.bar import InitClass
from foo.bar.baz.mod import SuchNestedMuchWow, AnotherNestedMuchWow

print(SuchNestedMuchWow().class_item)
#^^^^ reference  python-stdlib 3.13 builtins/print().
#     ^^^^^^^^^^^^^^^^^ reference local 0
print(AnotherNestedMuchWow().other_item)
#^^^^ reference  python-stdlib 3.13 builtins/print().
#     ^^^^^^^^^^^^^^^^^^^^ reference local 1
print(InitClass().init_item)
#^^^^ reference  python-stdlib 3.13 builtins/print().
#     ^^^^^^^^^ reference local 2

