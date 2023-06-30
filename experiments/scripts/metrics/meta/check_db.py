from schema import *

connect()

import json

if __name__ == "__main__":
    dataf = sys.argv[2] # argv 1 is the database name already loaded when schema.py is imported
    with open(dataf) as jsonfile:
        data = json.load(jsonfile)

        # The count of Wasm records in the db should be the same as the number of variants
        print("Number of variants: ", len(data))
        print("Number of wasm records: ", Wasm.select().count())

        assert len(data) == Wasm.select().count()