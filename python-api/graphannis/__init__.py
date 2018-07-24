__all__ = ['cs', 'query']

from graphannis._ffi import ffi

# TODO: make this work on other platforms (Windows/Mac)
CAPI = ffi.dlopen('libgraphannis_capi.so')