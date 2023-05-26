#!/usr/bin/env python

import sys
import os

DIRNAME = os.path.abspath(os.path.dirname(__file__))
#WASMTIME = os.environ.get('WASMTIME', 'wasmtime')
WASMTIME = "/home/jacarte/Documents/side/tawasco/host_based/host_single/target/release/host_single"


sys.path.insert(0, os.path.join(DIRNAME, '../deadpool'))

from deadpool_dca import *

def processinput(iblock, blocksize):
    p='%0*x' % (2*blocksize, iblock)
    return (None, [p[j*2:(j+1)*2] for j in range(len(p)//2)])

def processoutput(output, blocksize):
    print output
    return int(output, 16)

filters=[Filter('mem_addr1_r1', ['R'], lambda stack_range, addr, size, data: addr > 0x7ffffff6ec00 and size == 1, lambda addr, size, data: addr & 0xFF, '<B'),
         Filter('mem_data_r1',  ['R'], lambda stack_range, addr, size, data: addr > 0x7ffffff6ec00 and size == 1, lambda addr, size, data: data, '<B'),
         Filter('mem_stack_w1',  ['W'], lambda stack_range, addr, size, data: addr < 0x7ffffff6ec00 and size == 1, lambda addr, size, data: data, '<B')]

# T=TracerGrind('./DemoKey_table_encrypt', processinput, processoutput, ARCH.amd64, 16, filters=filters)
T=TracerPIN('%s %s'%(WASMTIME, "kryptologic.cwasm"), processinput, processoutput, ARCH.amd64, 16, filters=filters)

T.run(200)

bin2daredevil(keywords=filters, config={'algorithm':'AES', 'position':'LUT/AES_AFTER_SBOX'})
