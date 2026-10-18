# < definition scip-python python snapshot-util 0.1 simple/__init__:

# format-options: showRanges
def simple(x):
#   ^^^^^^ definition  snapshot-util 0.1 simple/simple().
#          ^ definition  snapshot-util 0.1 simple/simple().(x)
  return x
#        ^ reference  snapshot-util 0.1 simple/simple().(x)

