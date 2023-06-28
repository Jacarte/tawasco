import sys
import os
import subprocess
import json
import zipfile as zzip
import shutil
'''
    Extracts the mutation hsitory of a variant.
    Returns: the number of mutations and its history (the sequence of mutator names)
'''
def extract_mutation_history(variantname, zipf, funcname):
    # The parent relationship is stored as the name of the file f"{bname1}->{bname2} ({seed}).logs.txt"
    # First find a file in the zip file with that name where currentfile is variant
    # Then extract the log file to the out folder

    def get_tree(node, outfolder, zipfile, step = 0):
        print(node)
        # extract the log and continue to the parent until original is met
        
        name = node.replace(".logs.txt", "")
        files = name.split(" ")
        parent, _ = files.split("->")

        if parent == "original":
            return

        # extract file to outfolder
        return get_tree(parent, outfolder, zipfile)

    variantname = os.path.basename(variantname)
    # First itearte over the zipfile
    for f in zipf.namelist():
        # get the name of the file
        name = os.path.basename(f)
        # check if it is a log file
        if name.endswith(".logs.txt"):
            # print(name)
            # extract parent and file
            name = name.replace(".logs.txt", "")
            files, seed = name.split(" ")
            bname1, bname2 = files.split("->")
            #bname1 = bname1.replace(".wasm", "").strip()
            #bname2 = bname2.replace(".wasm", "")
            # check if the file is the variant
            # print(bname1, bname2, variantname)
            if bname2 == variantname:
                print(name)
                # Create a folder here with the name of the variant
                os.makedirs(os.path.join(f"equals/{os.path.dirname(f)}", variantname), exist_ok=True)
                
                source = zipf.open(f"{f}")
                target = open(f"equals/{f}", "wb")
                with source, target:
                    shutil.copyfileobj(source, target)

'''
    Provides some data about the variants population.
    Returns: the distribution of stacked mutations, the number of first level mutations (only one mutation applied to the original program), the largest stacked mutations, size distribution of the variants.
'''
def get_population_metadata(f):
    # download the variants from the bucket
    if not os.path.exists(f"{f}.variants.zip"):
        s = subprocess.check_output(["mc", "cp", f"exp/wasm-mutate/variants/{f}.c/variants.zip", f"{f}.variants.zip"])

    # traverse the zip file
    with zzip.ZipFile(f"{f}.variants.zip", "r") as zip_ref:
        # read all files ending in .wasm
        files = [f for f in zip_ref.namelist() if f.endswith(".wasm")]
        # get the hash of the file content
        hashes = [zip_ref.getinfo(f).CRC for f in files]
        print("Number of variants: ", len(hashes))
        # turn the hashes into hash and counter
        meta = {}

        for h, c in zip(hashes, files):
            if h not in meta:
                meta[h] = {"count": 0,  "files": []}
            meta[h]["count"] += 1
            meta[h]["files"].append(c)

        print("Number of unique variants: ", len(set(meta)))

        with open(f"results.json", "w") as jsonfile:
            json.dump(meta, jsonfile, indent=4)
        
        # extract some files that are equal
        for h in meta:
            if len(meta[h]['files']) > 1:
                # extrat them
                for fname in meta[h]['files']:
                    # extract only the file
                    source = zip_ref.open(fname)
                    os.makedirs(os.path.join(f"out/{f}"), exist_ok=True)
                    target = open(os.path.join(f"out/{f}", os.path.basename(fname)), "wb")
                    with source, target:
                        shutil.copyfileobj(source, target)

                    # Extract the mutation history of all variants here
                    # hist = extract_mutation_history(fname, zip_ref, f)
                break
    # remove the file
    # os.remove(f"{f}.variants.zip")



'''
    Checks for the preservation of the variants. 
    The first  argument is the name of a program. It downloads the data and perform the DTW pairwise comparison.
    The second argument is the data representation: Wasm, x86 wasmtime, x86 v8 and Bynaryen opt.

    The result is saved as a json file.
'''
def check_preservation(f, representation):
    pass

if __name__ == "__main__":
    os.makedirs("out", exist_ok=True)
    # plot_data("result.json")
    get_population_metadata(sys.argv[1].split(".")[0])