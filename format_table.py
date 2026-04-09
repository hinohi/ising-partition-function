"""data/{n}x{n}.txt を読んで README 用のテーブルを出力する"""
import sys
from collections import defaultdict

n = int(sys.argv[1])
nn = n * n
e_size = 2 * nn + 1
m_size = nn + 1

data = defaultdict(int)
with open(f"data/{n}x{n}.txt") as f:
    for line in f:
        e, m, c = line.split()
        data[(int(e), int(m))] = int(c)

# magnetization values: -nn, -nn+2, ..., nn-2, nn
mags = list(range(-nn, nn + 1, 2))
# energy values: -2*nn, -2*nn+2, ..., 2*nn
enes = list(range(-2 * nn, 2 * nn + 1, 4))

# compute column widths
widths = []
for m in mags:
    w = max(len(str(m)), 3)
    for e in enes:
        w = max(w, len(str(data.get((e, m), 0))))
    widths.append(w)

# header
header = "  \\m"
for m, w in zip(mags, widths):
    header += f" {m:>{w}}"
print(header)

sep = " e \\"
for w in widths:
    sep += "-" * (w + 1)
print(sep)

# rows (skip odd energy indices = all zero)
for e in enes:
    if (e + 2 * nn) % 2 == 1:
        continue
    row = f"{e:4}|"
    for m, w in zip(mags, widths):
        row += f" {data.get((e, m), 0):>{w}}"
    print(row)
