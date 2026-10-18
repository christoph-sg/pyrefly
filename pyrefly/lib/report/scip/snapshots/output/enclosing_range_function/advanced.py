# < definition scip-python python snapshot-util 0.1 advanced/__init__:

# format-options: showRanges
def decorator(func):
    def wrapper(*args, **kwargs):
        return func(*args, **kwargs)
    return wrapper

@decorator
def func(x: float) -> float:
    test = x

    return test

