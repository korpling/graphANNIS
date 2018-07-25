# Gives access to a (sub)-graph stored in graphANNIS and allows to map it to networkX

import networkx as nx
from graphannis import CAPI, ffi

def _get_node_labels(nID, db):
    result = dict()
    annos = CAPI.annis_graph_node_labels(db, nID[0])

    for i in range(CAPI.annis_vec_annotation_size(annos)):
        a = CAPI.annis_vec_annotation_get(annos, i)
        ns = CAPI.annis_graph_str(db, a.key.ns)
        name = CAPI.annis_graph_str(db, a.key.name)
        val = CAPI.annis_graph_str(db, a.val)

        if ns != ffi.NULL and name != ffi.NULL:
            if ns == ffi.NULL:
                qname = ffi.string(name).decode('utf-8')
            else:
                qname = ffi.string(ns).decode('utf-8') + '::' + ffi.string(name).decode('utf-8')
        result[qname] = ffi.string(val).decode('utf-8')
    return result

def _map_node(G, nID, db):
    labels = _get_node_labels(nID, db)

    if labels['annis::node_name'] != None:
        n = labels['annis::node_name']
        
        G.add_node(n)

def map_graph(db):
    G = nx.DiGraph()
    if db == ffi.NULL:
        return G
        
    # create all new nodes
    it_nodes = CAPI.annis_graph_nodes_by_type(db, b'node')
    if it_nodes != ffi.NULL:
        nID = CAPI.annis_iter_nodeid_next(it_nodes)
        while nID != ffi.NULL:

            _map_node(G, nID, db)

            nID = CAPI.annis_iter_nodeid_next(it_nodes)

    return G