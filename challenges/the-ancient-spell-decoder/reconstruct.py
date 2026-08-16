import sys
from collections import Counter

def build_trie(words):
    trie = {'cnt': 0, 'ch': {}}
    for w in words:
        n = trie
        n['cnt'] += 1
        for c in w:
            n = n['ch'].setdefault(c, {'cnt': 0, 'ch': {}})
            n['cnt'] += 1
    return trie

def solve(words, expected, total_len):
    """Return list of (R, S) such that removing word R from the dictionary
    lets the scroll S (length total_len) reproduce the expected counts."""
    trie = build_trie(words)
    first = expected[0]
    c = Counter(w[0] for w in words)
    cands = {ch for ch, cnt in c.items() if cnt == first + 1}
    print("candidate first letters to remove from:", sorted(cands))
    nz = expected.index(0) if 0 in expected else len(expected)
    nonzero = expected[:nz]
    sols = []
    for R in words:
        if R[0] not in cands:
            continue
        S_prefix = []
        def dfs(node, depth, prefix):
            if depth == len(nonzero):
                return True
            want = nonzero[depth]
            for ch in sorted(node['ch']):
                eff = node['ch'][ch]['cnt']
                if len(R) > depth and prefix == R[:depth] and R[depth] == ch:
                    eff -= 1
                if eff == want:
                    S_prefix.append(ch)
                    if dfs(node['ch'][ch], depth + 1, prefix + ch):
                        return True
                    S_prefix.pop()
            return False
        if dfs(trie, 0, ""):
            p = "".join(S_prefix)
            if nz == len(expected):
                sols.append((R, p))
                continue
            n = trie
            for ch in p:
                n = n['ch'][ch]
            z = None
            for ch in 'abcdefghijklmnopqrstuvwxyz':
                if ch not in n['ch']:
                    z = ch
                    break
            if z is None:
                continue
            S = p + z + 'a' * (total_len - len(p) - 1)
            sols.append((R, S))
    return sols

if __name__ == '__main__':
    words_file, expected_file, total_len, out = sys.argv[1], sys.argv[2], int(sys.argv[3]), sys.argv[4]
    words = [w.strip() for w in open(words_file) if w.strip()]
    expected = [int(x) for x in open(expected_file).read().split()]
    print(f"words={len(words)} expected_len={len(expected)}")
    sols = solve(words, expected, total_len)
    print("solutions:", len(sols))
    for R, S in sols[:20]:
        print("  R=", R, " S=", S)
    if sols:
        R, S = sols[0]
        with open(out, 'w') as f:
            f.write(str(len(words) - 1) + "\n")
            for w in words:
                if w != R:
                    f.write(w + "\n")
            f.write(S + "\n")
        print("wrote", out, "K=", len(words) - 1, "removed:", R)
