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


def estimate_stride_size(f, sizes = []):
    # read the file
    if not sizes:
        sizes = [ 256, 512, 1024, 2048, 4096, 2*4096]
    
    for s in sizes:
        NEWFILE = []
        lines = open(f, 'r').readlines()
        REP = False
        i = 0
        while i < len(lines):
            line = lines[i]
            if line.strip() == '//////// STRIDE SIZE START':
                # the next line is replaced with the stride size
                NEWFILE.append('//////// STRIDE SIZE START\n')
                NEWFILE.append(f"pub const STRIDE: usize = {s};\n")
                # Skip the next line
                i += 1
            else:
                NEWFILE.append(line)
            i += 1
        
        # write the new file
        with open(f, 'w') as fw:
            for line in NEWFILE:
                fw.write(line)
        
        # Now compile it
        ch = subprocess.run(['make'], stderr=subprocess.DEVNULL, stdout=subprocess.DEVNULL)
        #if ch.returncode != 0:

        # execute the eviction bin
        AVG = 0
        for t in [1000]:

            for j in range(10):
                ch = subprocess.run(['./target/release/eviction'], stdout=subprocess.PIPE, env = {
                    'TRIES': f'{t}',
                })
                r, c = BER("My password", ch.stdout.decode())
                AVG += 1 - r
                print(s, t,  ch.stdout.decode(), 1-r, 11-c)
        print(s, AVG/80)
        
        





if __name__ == "__main__":
    f = sys.argv[1]
    estimate_stride_size(f)