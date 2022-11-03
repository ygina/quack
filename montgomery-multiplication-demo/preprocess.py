import crt

# https://primes.utm.edu/lists/2small/0bit.html
N = 2**63 - 25
R = 2**63
RInv, NInv = crt.invert(R, N)
NPrime = R - NInv

assert (R * RInv) % N == 1
assert (N * NInv) % R == 1
assert (N * NPrime) % R == (R - 1)

print(f"N = {N}ull,")
print(f"R = {R}ull,")
print(f"RInv = {RInv}ull,")
print(f"NegNInv = {NPrime}ull;")
