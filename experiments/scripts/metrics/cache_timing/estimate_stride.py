import sys
import os
import re
import subprocess

def BER(k1, k2):
    MISS = 0
    for i, c in enumerate(k2):
        if c != k1[i]:
            MISS += 1
    return MISS / len(k1), MISS


def estimate_stride_size(f, t):
    # read the file

    AVG = 0
    NUM = 100
    print(t, NUM)
    for j in range(NUM):
        ch = subprocess.run(['host', f], stdout=subprocess.PIPE, env = {
            'TRIES': f'{t}',
        })
        r, c = BER("My password", ch.stdout.decode())
        AVG += 1 - r
        print(t,  ch.stdout.decode(), 1-r, 11-c)
        sys.stderr.write("%s %s %s\n"%(t,  ch.stdout.decode(), 1-r, 11-c))
    print(AVG/NUM)
    return AVG/NUM
        
        





if __name__ == "__main__":
    f = sys.argv[1]
    size = int(sys.argv[2])
    r = estimate_stride_size(f, size)
    sys.stderr.write("%s\n"%(r, ))