from graphannis import CAPI
from graphannis._ffi import ffi

class CorpusStorageManager:
    def __init__(self, db_dir='data/', use_parallel=True):
        self.__cs = CAPI.annis_cs_new(db_dir.encode('utf-8'), use_parallel)

    def __enter__(self):
        return self
    
    def __exit__(self, exc_type, exc_value, traceback):
        CAPI.annis_free(self.__cs)

    def list(self):
        orig = CAPI.annis_cs_list(self.__cs)
        orig_size = int(CAPI.annis_vec_str_size(orig))

        copy = []
        for idx, val in enumerate(range(orig_size)):
            corpus_name = ffi.string(CAPI.annis_vec_str_get(orig, idx))
            copy.append(corpus_name.decode('utf-8'))
        return copy
    
    def count(self, corpora, query_as_json):
        result = int(0)
        for c in corpora:
            result = result + CAPI.annis_cs_count(self.__cs, c.encode('utf-8'), query_as_json.encode('utf-8'))

        
        return result


    