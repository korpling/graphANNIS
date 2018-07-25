from .common import CAPI
from ._ffi import ffi
from .graph import map_graph

class CSException(Exception):
    def __init__(self, m : str):
        self.message = m

    def __str__(self):
        return self.message


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

    def find(self, corpora, query_as_json, offset=0, limit=10):
        result = []
        for c in corpora:
            vec = CAPI.annis_cs_find(self.__cs, c.encode('utf-8'), query_as_json.encode('utf-8'), offset, limit)
            vec_size = CAPI.annis_vec_str_size(vec)
            for i in range(vec_size):
                result_str = ffi.string(CAPI.annis_vec_str_get(vec, i)).decode('utf-8')
                result.append(result_str)
        return result

    def subgraph(self, corpus_name, node_ids, ctx_left=0, ctx_right=0):
        c_node_ids = CAPI.annis_vec_str_new()
        for nid in node_ids:
            CAPI.annis_vec_str_push(c_node_ids, nid.encode('utf-8'))
        
        db = CAPI.annis_cs_subgraph(self.__cs, corpus_name.encode('utf-8'), c_node_ids, ctx_left, ctx_right)

        G = map_graph(db)

        CAPI.annis_free(db)
        CAPI.annis_free(c_node_ids)

        return G

    def apply_update(self, corpus_name, update):
        """ Atomically apply update (add/delete nodes, edges and labels) to the database

        >>> from graphannis.cs import CorpusStorageManager
        >>> from graphannis.graph import GraphUpdate 
        >>> with CorpusStorageManager() as cs:
        ...     with GraphUpdate() as g:
        ...         g.add_node('n1')
        ...         cs.apply_update('test', g)
        """ 
        
        result = CAPI.annis_cs_apply_update(self.__cs,
        corpus_name.encode('utf-8'), update.get_instance())

        if result != ffi.NULL:
            msg = ffi.string(CAPI.annis_error_get_msg(result)).decode('utf-8')
            CAPI.annis_free(result)
            raise CSException(msg)

    def delete_corpus(self, corpus_name):
        """ Delete a corpus from the database

        >>> from graphannis.cs import CorpusStorageManager
        >>> with CorpusStorageManager() as cs:
        ...     cs.delete_corpus('test')
        """ 
        CAPI.annis_cs_delete(self.__cs, corpus_name.encode('utf-8'))