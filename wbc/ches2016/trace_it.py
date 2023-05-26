#!/usr/bin/env python

import sys
import os
import re

DIRNAME = os.path.abspath(os.path.dirname(__file__))
#WASMTIME = os.environ.get('WASMTIME', 'wasmtime')
WASMTIME = "/home/jacarte/Documents/side/tawasco/host_based/host_single/target/release/host_single"

sys.path.insert(0, os.path.join(DIRNAME, '../deadpool'))
from deadpool_dca import *

def processinput(iblock, blocksize):
    p='%0*x' % (2*blocksize, iblock)
    return (None, [p[j*2:(j+1)*2] for j in range(len(p)/2)])

def processoutput(output, blocksize):
    print output
    return int(''.join([x for x in output.split('\n') if x.find('OUTPUT')==0][0][10:].split(' ')), 16)

# Change the path to the target binary:
# TODO, we need to fix the address
# T=TracerGrind('%s %s'%(WASMTIME, "wb_challenge.cwasm"), processinput, processoutput, ARCH.amd64, 16,  addr_range='default')
T=TracerPIN('%s %s'%(WASMTIME, "wb_challenge.wasm"), processinput, processoutput, ARCH.amd64, 16,  addr_range='4', stack_range="0x20080000-0x7fffffff")
# T=TracerPIN('%s %s'%(WASMTIME, "mem_test.cwasm"), processinput, processoutput, ARCH.amd64, 16,  addr_range='default')

#T=TracerGrind('%s %s'%(WASMTIME, "wb_challenge.cwasm"), processinput, processoutput, ARCH.amd64, 16,  addr_range='0x108000-0x130000')

T.run(1)
bin2daredevil(configs={'attack_sbox':   {'algorithm':'AES', 'position':'LUT/AES_AFTER_SBOX'},
                       'attack_multinv':{'algorithm':'AES', 'position':'LUT/AES_AFTER_MULTINV'}})
