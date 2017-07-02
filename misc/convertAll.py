#!/usr/bin/env python3

import os
from subprocess import call;

for d in os.listdir('relannis/'):
    print("Checking " + d)
    if(os.path.isdir('relannis/' + d)):
        print("Converting " + d)
        call(["build/annis_runner", "import", 'relannis/' + d, 'data/' + d])

