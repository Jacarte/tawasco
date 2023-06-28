import sys
import os
import re
import subprocess

HOST = os.environ.get('HOST_SINGLE', 'host')

def BER(k1, k2):
    MISS = 0
    for i, c in enumerate(k2):
        if c != k1[i]:
            MISS += 1
    return MISS / len(k1), MISS


def estimate_stride_size(f, t, t2, SECRET="My password"):
    # read the file

    AVG = 0
    NUM = 100
    print(t, NUM)
    for j in range(NUM):
        ch = subprocess.run([HOST, f], stdout=subprocess.PIPE, env = {
            'TRIES': f'{t}',
            'TRIALS': f'{t2}',
            **os.environ
        })
        r, c = BER(SECRET, ch.stdout.decode())
        AVG += 1 - r
        print(t,  ch.stdout.decode(), 1-r, 11-c)
        sys.stderr.write("%s %s %s\n"%(t,  ch.stdout.decode(), 1-r, 11-c))
    print(AVG/NUM)
    return AVG/NUM
        
        





if __name__ == "__main__":
    f = sys.argv[1]
    size = int(sys.argv[2])
    t2 = int(sys.argv[3])
    SECRET = sys.argv[4]
    r = estimate_stride_size(f, size, t2, SECRET)
    sys.stderr.write("%s\n"%(r, ))
    print(r)