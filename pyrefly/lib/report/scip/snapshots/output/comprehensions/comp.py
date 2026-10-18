# < definition scip-python python snapshot-util 0.1 comp/__init__:

def something(x):
#   ^^^^^^^^^ definition  snapshot-util 0.1 comp/something().
#             ^ definition  snapshot-util 0.1 comp/something().(x)
    return 2 * x

_ = [x for x in "should be local 0"]
# definition  snapshot-util 0.1 comp/_.
#          ^ definition local 0
_ = [something(x) for x in "should be local 1"]
# definition  snapshot-util 0.1 comp/_.
#                     ^ definition local 1
_ = [something(x) for x in "should be local 2" if x == "s"]
# definition  snapshot-util 0.1 comp/_.
#                     ^ definition local 2
_ = {k: x for (k, x) in enumerate(range(10))}
# definition  snapshot-util 0.1 comp/_.
#              ^ definition local 3
#                 ^ definition local 4


asdf = (var for var in [1, 2, 3] if var % 2 == 0)
#^^^ definition  snapshot-util 0.1 comp/asdf.
#               ^^^ definition local 5
for var in asdf:
#   ^^^ definition  snapshot-util 0.1 comp/var.
    print(var)

# TODO: ListComprehensions with if
# TODO: Dictionary comprehensions


