#!/usr/bin/env python3

import os
from subprocess import call;

for d in os.listdir('relannis/'):
    print("Checking " + d)
    if(os.path.isdir('relannis/' + d)):
        print("Converting " + d)
        call("target/release/annis data/ --cmd 'import relannis/" + d + " " + d + "'", shell=True)



