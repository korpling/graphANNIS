from _ffi import ffi

class CorpusStorageManager:
    def __init__(self, db_dir='data/', use_parallel=True):
        # TODO: make this work on other platforms (Windows/Mac)
        self.C = ffi.dlopen('libgraphannis_capi.so')

        self.__cs = self.C.annis_cs_new(db_dir.encode('utf-8'), use_parallel)

    def __enter__(self):
        return self
    
    def __exit__(self, exc_type, exc_value, traceback):
        self.C.annis_cs_free(self.__cs)

    