# Gives access to a (sub)-graph stored in graphANNIS and allows to map it to networkX

import networkx as nx
from .common import CAPI, ffi

def _get_node_labels(nID, db):
    result = dict()
    annos = CAPI.annis_graph_node_labels(db, nID)

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

        CAPI.annis_str_free(ns)
        CAPI.annis_str_free(name)
        CAPI.annis_str_free(val)

    return result

def _get_edge_labels(db, edge_ptr, component_ptr):
    result = dict()

    edge = {'source' : edge_ptr.source, 'target' : edge_ptr.target}

    annos = CAPI.annis_graph_edge_labels(db, edge, component_ptr)

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

        CAPI.annis_str_free(ns)
        CAPI.annis_str_free(name)
        CAPI.annis_str_free(val)
    return result

def _map_node(G, db, nID):
    labels = _get_node_labels(nID, db)
    
    G.add_node(nID)
    for key, value in labels.items():
        G.nodes[nID][key] = value

def _map_edge(G, db, edge_ptr, component_ptr):
    labels = _get_edge_labels(db, edge_ptr, component_ptr)

    G.add_edge(edge_ptr.source, edge_ptr.target)
    for key, value in labels.items():
        G.edges[edge_ptr.source, edge_ptr.target][key] = value
    # always add the component name and type as attribute
    component_name = CAPI.annis_component_name(component_ptr)
    G.edges[edge_ptr.source, edge_ptr.target]['annis::component_name'] = ffi.string(component_name).decode('utf-8')
    CAPI.annis_str_free(component_name)

    component_type_enum = CAPI.annis_component_type(component_ptr)
    component_type = None
    if component_type_enum == CAPI.Coverage:
        component_type = 'Coverage'
    elif component_type_enum == CAPI.InverseCoverage:
        component_type = 'InverseCoverage'
    elif component_type_enum == CAPI.Dominance:
        component_type = 'Dominance'
    elif component_type_enum == CAPI.Pointing:
        component_type = 'Pointing'
    elif component_type_enum == CAPI.Ordering:
        component_type = 'Ordering'
    elif component_type_enum == CAPI.LeftToken:
        component_type = 'LeftToken'
    elif component_type_enum == CAPI.RightToken:
        component_type = 'RightToken'
    elif component_type_enum == CAPI.PartOfSubcorpus:
        component_type = 'PartOfSubcorpus'

    if component_type != None:
        G.edges[edge_ptr.source, edge_ptr.target]['annis::component_type'] = component_type

def map_graph(db):
    G = nx.DiGraph()
    if db == ffi.NULL:
        return G
        
    # create all new nodes
    it_nodes = CAPI.annis_graph_nodes_by_type(db, b'node') # TODO: should we also map the corpus graph?
    if it_nodes != ffi.NULL:
        ptr_nID = CAPI.annis_iter_nodeid_next(it_nodes)
        while ptr_nID != ffi.NULL:

            nID = ptr_nID[0]
            _map_node(G, db, nID)

            CAPI.annis_free(ptr_nID)
            ptr_nID = CAPI.annis_iter_nodeid_next(it_nodes)

    CAPI.annis_free(it_nodes)

    # find all components of the graph
    components = CAPI.annis_graph_all_components(db)
    component_size = CAPI.annis_vec_component_size(components)

    # for each node of the graph, find all outgoing edges and add them
    for n in list(G):
        for c_idx in range(component_size):
            component_ptr = CAPI.annis_vec_component_get(components, c_idx)
            outEdges = CAPI.annis_graph_outgoing_edges(db, n, component_ptr)
            for edge_idx in range(CAPI.annis_vec_edge_size(outEdges)):
                edge_ptr = CAPI.annis_vec_edge_get(outEdges, edge_idx)

                _map_edge(G, db, edge_ptr, component_ptr)
            CAPI.annis_free(outEdges)


    CAPI.annis_free(components)

    return G

class GraphUpdate:

    def __init__(self):
        self.__instance = CAPI.annis_graphupdate_new()

    def __enter__(self):
        return self
    
    def __exit__(self, exc_type, exc_value, traceback):
        CAPI.annis_free(self.__instance)

    def get_instance(self):
        return self.__instance

    def add_node(self, node_name, node_type='node'):
        """Add a named node to the graph

        >>> from graphannis.graph import GraphUpdate
        >>> with GraphUpdate() as g:
        ...     g.add_node('n1')
        ...
        """
        CAPI.annis_graphupdate_add_node(self.__instance,
        node_name.encode('utf-8'), 
        node_type.encode('utf-8'))
    
    def add_node_label(self, node_name, anno_ns, anno_name, anno_value):
        """Add a label to an existing node to the graph
        
        >>> from graphannis.graph import GraphUpdate
        >>> with GraphUpdate() as g:
        ...     g.add_node('n1')
        ...     g.add_node_label('n1', 'mynamespace', 'myname', 'myvalue')
        ...
        """
        CAPI.annis_graphupdate_add_node_label(self.__instance, 
        node_name.encode('utf-8'),
        anno_ns.encode('utf-8'),
        anno_name.encode('utf-8'), 
        anno_value.encode('utf-8'))

    def delete_node_label(self, node_name, anno_ns, anno_name):
        """Delete an existing label from an existing node
        
        >>> from graphannis.graph import GraphUpdate
        >>> with GraphUpdate() as g:
        ...     g.add_node('n1')
        ...     g.add_node_label('n1', 'mynamespace', 'myname', 'myvalue')
        ...     g.delete_node_label('n1', 'mynamespace', 'myname')
        ...
        """
        CAPI.annis_graphupdate_delete_node_label(self.__instance, 
        node_name.encode('utf-8'),
        anno_ns.encode('utf-8'), anno_name.encode('utf-8'))

    def add_edge(self, source_node, target_node, layer, 
    component_type, component_name):
        """Add an edge between two existing nodes.
        
        >>> from graphannis.graph import GraphUpdate
        >>> with GraphUpdate() as g:
        ...     g.add_node('n1')
        ...     g.add_node('n2')
        ...     g.add_edge('n1', 'n2', 'mylayer', 'Pointing', 'dep')
        ...
        """
        CAPI.annis_graphupdate_add_edge(self.__instance, 
        source_node.encode('utf-8'), target_node.encode('utf-8'), 
        layer.encode('utf-8'), 
        component_type.encode('utf-8'), 
        component_name.encode('utf-8'))

    def delete_edge(self, source_node, target_node, layer, component_type,
    component_name):
        """Delete an existingedge between two nodes.
        
        >>> from graphannis.graph import GraphUpdate
        >>> with GraphUpdate() as g:
        ...     g.add_node('n1')
        ...     g.add_node('n2')
        ...     g.add_edge('n1', 'n2', 'mylayer', 'Pointing', 'dep')
        ...     g.delete_edge('n1', 'n2', 'mylayer', 'Pointing', 'dep')
        ...
        """
        CAPI.annis_graphupdate_add_edge(self.__instance, 
        source_node.encode('utf-8'), target_node.encode('utf-8'), 
        layer.encode('utf-8'), 
        component_type.encode('utf-8'), 
        component_name.encode('utf-8'))

    def add_edge_label(self, source_node, target_node, 
    layer, component_type, component_name,
    anno_ns, anno_name, anno_value):
        """Add a label to an existing edge 

        >>> from graphannis.graph import GraphUpdate
        >>> with GraphUpdate() as g:
        ...     g.add_node('n1')
        ...     g.add_node('n2')
        ...     g.add_edge('n1', 'n2', 'mylayer', 'Pointing', 'dep')
        ...     g.add_edge_label('n1', 'n2', 'mylayer', 'Pointing', 'dep',
        ...     'myns', 'myanno', 'annoval')
        ...
        """
        CAPI.annis_graphupdate_add_edge_label(self.__instance, 
        source_node.encode('utf-8'), target_node.encode('utf-8'), 
        layer.encode('utf-8'), component_type.encode('utf-8'), component_name.encode('utf-8'),
        anno_ns.encode('utf-8'), anno_name.encode('utf-8'),
        anno_value.encode('utf-8'))

    def delete_edge_label(self, source_node, target_node, 
    layer, component_type, component_name,
    anno_ns, anno_name):
        """Delete a label from an edge
        
        >>> from graphannis.graph import GraphUpdate
        >>> with GraphUpdate() as g:
        ...     g.add_node('n1')
        ...     g.add_node('n2')
        ...     g.add_edge('n1', 'n2', 'mylayer', 'Pointing', 'dep')
        ...     g.add_edge_label('n1', 'n2', 'mylayer', 'Pointing', 'dep',
        ...     'myns', 'myanno', 'annoval')
        ...     g.delete_edge_label('n1', 'n2', 'mylayer', 'Pointing', 'dep',
        ...     'myns', 'myanno')
        ...
        """
        CAPI.annis_graphupdate_delete_edge_label(self.__instance, 
        source_node.encode('utf-8'), target_node.encode('utf-8'), 
        layer.encode('utf-8'), component_type.encode('utf-8'), component_name.encode('utf-8'),
        anno_ns.encode('utf-8'), anno_name.encode('utf-8'))
