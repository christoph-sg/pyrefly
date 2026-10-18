# < definition scip-python python snapshot-util 0.1 `src.long_importer`/__init__:

import foo.bar.baz.mod

print(foo.bar.baz.mod.SuchNestedMuchWow)
#^^^^ reference  python-stdlib 3.13 builtins/print().
#     ^^^ reference local 0

