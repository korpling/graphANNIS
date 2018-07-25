import os, os
from ._ffi import ffi

# TODO: make this work on other platforms (Windows/Mac)
parent_dir = os.path.normpath(os.path.realpath(__file__) + '/..')
search_dirs = [
    parent_dir + '/data/linux-x86-64/libgraphannis_capi.so',
    parent_dir + '/../package/linux-x86-64/libgraphannis_capi.so',
    parent_dir + '/../../target/release/libgraphannis_capi.so'
]

for d in search_dirs:
    d = os.path.normpath(d)
    if os.path.exists(d):
        shared_obj_file = d
        break

if shared_obj_file == None:
    # let the system search for the shared library
    shared_obj_file = 'libgraphannis_capi.so'    

CAPI = ffi.dlopen(shared_obj_file)