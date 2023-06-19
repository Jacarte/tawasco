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


def change_stride_size(f, s):
    # read the file
    
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
        


if __name__ == "__main__":
    f = sys.argv[1]
    s = int(sys.argv[2])
    change_stride_size(f, s)