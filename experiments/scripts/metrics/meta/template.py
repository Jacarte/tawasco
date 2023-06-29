from schema import *

connect()

'''
    Get the blake3 hash of the binaryen wasm file.
'''
def get_wasm_blake3_hash(zip, name):
    bytes = zip.read(name)
    return blake3(bytes).hexdigest()


'''
    Provides some data about the variants population.
    Returns: the distribution of stacked mutations, the number of first level mutations (only one mutation applied to the original program), the largest stacked mutations, size distribution of the variants.
'''
def check_preservation(f):
    # download the db

    if not os.path.exists(f"{f}.variants.zip"):
        s = subprocess.check_output(["mc", "cp", f"exp/wasm-mutate/variants/{f}.c/variants.zip", f"{f}.variants.zip"])

    if not os.path.exists(f"{f}.c.db"):
        s = subprocess.check_output(["mc", "cp", f"exp/wasm-mutate/rq1/metas_raw_db/{f}.c.db", f"{f}.c.db"])
    
    with zzip.ZipFile(f"{f}.variants.zip", "r") as zip_ref:
        # get all Wasm instances
        wasm = Wasm.select()
        for w in wasm:
            print(w.name)
            exit(1)

    

if __name__ == "__main__":
    os.makedirs("out", exist_ok=True)
    # plot_data("result.json")
    check_preservation(NAME)