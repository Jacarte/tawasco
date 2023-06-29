from schema import *

init()

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

        # Get the parent first
        parent = filter(lambda x: x.endswith(f"{NAME}.c.wasm"), zip_ref.namelist())
        parent  = list(parent)[0]
        parent_inf = zip_ref.getinfo(parent)
        parent_hash = get_wasm_blake3_hash(zip_ref, parent)
        # Calculate the hash as blake3 of the bytestream

        parent_instance = Wasm.create(parent=None, optimized=False, hash=parent_hash, name=parent, original=None, parent_name=None)

        ORM_DATA = [

        ]
        I = 0
        for h, c in zip(hashes, files):
            if h not in meta:
                meta[h] = {"count": 0,  "files": []}
            # Create the wasm instance here
            if c != parent:
                hash = get_wasm_blake3_hash(zip_ref, c)
                # Get parent name from the file name 
                parent_name = c.split(".")[2]
                wasm_instance = Wasm(parent=None, optimized=False, hash=hash, name=c, original=parent_instance, parent_name=parent_name)
                ORM_DATA.append(wasm_instance)
                
                I += 1

                if I % 1000 == 999:
                    print("Saving", I)
                #wasm_instance = Wasm.create(parent=parent_instance, optimized=False, hash=h, name=c)

            meta[h]["count"] += 1
            meta[h]["files"].append(c)
        # Create the wasm instances here

        with db.atomic():
            size = (SQLITE_MAX_VARIABLE_NUMBER // len(ORM_DATA[0])) -1
            # remove one to avoid issue if peewee adds some variable
            for i in range(0, len(ORM_DATA), size):
                # table.insert_many(data[i:i+size]).upsert().execute()
                Wasm.bulk_create(ORM_DATA[i:i+size])

        ORM_DATA = []
        wasm_instances =  Wasm.select()
        # Create and instance of Mutation info for each one
        I = 0
        for wasm_instance in wasm_instances:
            info = MutationInfo(wasm=wasm_instance, description="Mutation")
            ORM_DATA.append(info)
            I += 1

            if I % 1000 == 999:
                print("Saving", I)
        
        with db.atomic():
            size = (SQLITE_MAX_VARIABLE_NUMBER // len(ORM_DATA[0])) -1
            # remove one to avoid issue if peewee adds some variable
            for i in range(0, len(ORM_DATA), size):
                # table.insert_many(data[i:i+size]).upsert().execute()
                MutationInfo.bulk_create(ORM_DATA[i:i+size])


        ORM_DATA = []
        I = 0
        #with db.atomic():
        for wasm_instance in wasm_instances:
            if wasm_instance.parent_name:
                parent_instance = Wasm.get(Wasm.hash == wasm_instance.parent_name)  
                # print("Parent", parent_instance)
                # wasm_instance.parent = parent_instance
                # Save the update for bulk insertion
                
                u = Wasm.update({Wasm.parent: 2}).where(Wasm.id == wasm_instance.id).execute()
                I += 1

                if I % 1000 == 999:
                    print("Saving", I)
        
        # Now update by the real path

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
    get_population_metadata(NAME)