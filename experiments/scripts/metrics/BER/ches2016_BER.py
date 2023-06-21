'''
    This script/module parses the daredevil result for the CHES2016 challenge in Wasm. It returns the BER for the attack_sbox and attack_multinv configurations.
'''

from BER import BER_OR_BITS, BER_OR_BYTES
import sys
import re

#2: 3.90801: a2ada1a6ada5a4abd2a21daea1a1e5a3
R = re.compile(r"(\d+): (\d+)(.\d+)?: ([0-9a-f]{32})")

def get_proposals(lines):
    proposals = []
    for line in lines:
        m = R.match(line)
        if m:
            proposals.append(m.group(4))

    return proposals

if __name__ == "__main__":
    sbox = sys.argv[1]
    multinv = sys.argv[2]

    sboxcontent = open(sbox, 'r').readlines()
    multinvcontent = open(multinv, 'r').readlines()

    sbox_proposals = get_proposals(sboxcontent)
    multinv_proposals = get_proposals(multinvcontent)

    assert len(sbox_proposals) == len(multinv_proposals)
    assert len(sbox_proposals) == 20

    original_key = "dec1a551f1eddec0de4b1dae5c0de511"

    c, t = BER_OR_BITS(original_key, sbox_proposals + multinv_proposals)

    print(c, t, 100*(1 - c/t))

    c, t = BER_OR_BYTES(original_key, sbox_proposals + multinv_proposals)

    print(c, t, 100*(1 - c/t))
