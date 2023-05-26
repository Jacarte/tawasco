
def get_bit_chain(key):
    bit_chain = []
    for i in range(0, len(key), 2):
        k = key[i:i+2]
        bit_chain.append(bin(int(k, 16))[2:].zfill(8))
    return bit_chain

'''
    Notice that this is counting the bits.
'''
def BER_OR_BITS(k1, proposals, mode = 'or'):

    chains = [ get_bit_chain(k) for k in proposals ]

    #print(chains)
    #if mode == 'or':
    # Or over all bins in the same position for all the proposals
    cumul = [0]*(len(chains[0]))
    for i in range(len(chains[0])):
        for j in range(0, len(chains)):
            cumul[i] = cumul[i] | int(chains[j][i], 2)
        cumul[i] = bin(cumul[i])[2:].zfill(8)
    
    # Now count the number of different bits with k1
    bitsk1 = get_bit_chain(k1)
    cumul = [ int(cumul[i], 2) ^ int(bitsk1[i], 2) for i in range(len(cumul)) ]
    # Count the number of 1s
    cumul = [ bin(cumul[i])[2:].zfill(8).count('1') for i in range(len(cumul)) ]
    cumul = sum(cumul)
    # print(cumul)
    
    total_bits = len(chains[0])*8

    return cumul, total_bits


def get_bytes(chain):
    return [ int(chain[i:i+2], 16) for i in range(0, len(chain), 2) ]

'''
    Notice that this is counting the bytes.
'''
def BER_OR_BYTES(k1, proposals, mode = 'or'):

    chains = [ get_bytes(k) for k in proposals ]
    orig = get_bytes(k1)
    # print(chains)
    #if mode == 'or':
    # Or over all bins in the same position for all the proposals
    cumul = [0]*(len(chains[0]))
    for i in range(len(chains[0])):
        for j in range(0, len(chains)):
            if chains[j][i] == orig[i]:
                cumul[i] = 1

    cumul = len(chains[0]) - sum(cumul) 
    # Now count the number of different bits with k1
    
    total_bytes = len(chains[0])

    return cumul, total_bytes

if __name__ == "__main__":
    
    bits = get_bit_chain('2b7e151628aed2a6abf7158809cf4f3c')
    # print(bits)

    c, t = BER_OR_BITS('2b7e151628aed2a6abf7158809cf4f3c', ['2b7e151628aed2a6abf7158809cf4f3c', '2b7e151628aed2a6abf7158809cf4f3c'])

    assert c == 0

    c, t = BER_OR_BITS('2b7e151628aed2a6abf7158809cf4f3c', ['2b7e151628aed2a6abf7158809cf4f3c', '107e151628aed2a6abf7158809cf4f3a'])

    print(c, t, 100*(1 - c/t))



    c, t = BER_OR_BYTES('2b7e151628aed2a6abf7158809cf4f3c', ['2b7e151628aed2a6abf7158809cf4f3c', '107e151628aed2a6abf7158809cf4f3a'])

    print(c, t, 100*(1 - c/t))