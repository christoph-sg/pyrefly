# < definition scip-python python snapshot-util 0.1 builtin_imports/__init__:

# format-options: showDocs

import re
#      ^^ reference  python-stdlib 3.13 re/__init__:
from typing import Callable, Optional
#    ^^^^^^ reference  python-stdlib 3.13 typing/__init__:
#                  ^^^^^^^^ reference  python-stdlib 3.13 typing/Callable.
#                            ^^^^^^^^ reference  python-stdlib 3.13 typing/Optional.

print(re, Callable, Optional)
#^^^^ reference  python-stdlib 3.13 builtins/print().
#     ^^ reference  python-stdlib 3.13 re/__init__:
#         ^^^^^^^^ reference  python-stdlib 3.13 typing/Callable.
#                   ^^^^^^^^ reference  python-stdlib 3.13 typing/Optional.

