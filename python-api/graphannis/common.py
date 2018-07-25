import os, os
from ._ffi import ffi

# TODO: make this work on other platforms (Windows/Mac)
parent_dir = os.path.normpath(os.path.realpath(__file__) + '/..')
if os.path.exists(parent_dir + '/data/linux-x86-64/libgraphannis_capi.so'):
    shared_obj_file = parent_dir + '/data/linux-x86-64/libgraphannis_capi.so'
else:
    shared_obj_file = os.path.normpath(parent_dir + '/../../target/release/libgraphannis_capi.so')

CAPI = ffi.dlopen(shared_obj_file)