#!/usr/bin/env python3

import sys
import csv
import os

if len(sys.argv) < 2:
    print("You have to give the query folder as argument.")
    exit(1)
query_folder = sys.argv[1]

fields = ["name", "aql", "corpus", "count"]

writer = csv.DictWriter(sys.stdout, fieldnames=fields)

writer.writeheader()

for subfolder in os.listdir(query_folder):
    subfolder_path = os.path.join(query_folder, subfolder)
    if os.path.isdir(subfolder_path):
        # the basename of a file is its name and the ending determines the column
        basenames = set()
        for file in os.listdir(subfolder_path):
            basenames.add(os.path.splitext(file)[0])
        for name in basenames:
            aql_file = os.path.join(subfolder_path, name + ".aql")
            count_file = os.path.join(subfolder_path, name + ".count")
            if os.path.isfile(aql_file) and os.path.isfile(count_file):
                row = {"corpus": subfolder, "name": name}
                with open(aql_file, "r") as f:
                    row["aql"] = f.read()
                with open(count_file, "r") as f:
                    row["count"] = int(f.read().strip())
                    
                writer.writerow(row)