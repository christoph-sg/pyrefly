# < definition scip-python python snapshot-util 0.1 walrus/__init__:

import re

some_list = [1, 2, 3, 4, 5, 100, 1000]
#^^^^^^^^ definition  snapshot-util 0.1 walrus/some_list.
pattern = "^[A-Za-z0-9_]*$"
#^^^^^^ definition  snapshot-util 0.1 walrus/pattern.
text = "Some_name123"
#^^^ definition  snapshot-util 0.1 walrus/text.

# if statement
if (n := len(some_list)) > 10:
#   ^ definition  snapshot-util 0.1 walrus/n.
    print(f"List is too long with {n} elements.")

if (match := re.search(pattern, text)) is not None:
#   ^^^^^ definition  snapshot-util 0.1 walrus/match.
    print("Found match:", match.group(0))


# comprehensions
def show_some_comprehension():
#   ^^^^^^^^^^^^^^^^^^^^^^^ definition  snapshot-util 0.1 walrus/show_some_comprehension().
    [print(x, root) for x in some_list if (x % 2 == 0) and (root := x**0.5) > 5]
#                       ^ definition local 0
#                                                           ^^^^ definition local 1


# while loop
while (line := input("Enter text: ")) != "quit":
#      ^^^^ definition  snapshot-util 0.1 walrus/line.
    print(f"You entered: {line}")


# if + comprehension
if any((any_n := num) < 0 for num in some_list):
#       ^^^^^ definition  snapshot-util 0.1 walrus/any_n.
#                             ^^^ definition local 2
    print(f"Negative number found: {any_n}")

