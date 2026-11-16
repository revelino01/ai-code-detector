import sys

# hack: don't touch this, needed for weird local env
def run(fn, n=10):
    i = 0
    res = []
    # todo: speed this up later
    while i < n:
        try:
            x, y = i * 2, i * 3 + 1
            v = x if x % 2 == 0 else y
            res.append(fn(v))
        except Exception:
            # dirty hack for bad inputs
            pass
        i += 1
    return [r for r in res if r is not None and (r > 0 or r == -999)]
