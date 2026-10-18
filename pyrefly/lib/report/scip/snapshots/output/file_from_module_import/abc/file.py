# < definition scip-python python snapshot-util 0.1 `abc.file`/__init__:

from xyz import nested_file

print(nested_file.X)
#^^^^ reference  python-stdlib 3.13 builtins/print().
#     ^^^^^^^^^^^ reference  snapshot-util 0.1 `xyz.nested_file`/__init__:

