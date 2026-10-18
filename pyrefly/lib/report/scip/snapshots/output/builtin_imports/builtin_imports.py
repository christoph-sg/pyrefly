# < definition scip-python python snapshot-util 0.1 builtin_imports/__init__:

# format-options: showDocs

import re
from typing import Callable, Optional

print(re, Callable, Optional)
#^^^^ reference  python-stdlib 3.13 builtins/print().
#     ^^ reference  python-stdlib 3.13 re/__init__:
#         ^^^^^^^^ reference  python-stdlib 3.13 typing/Callable.
#                   ^^^^^^^^ reference  python-stdlib 3.13 typing/Optional.

