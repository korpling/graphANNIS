import copy


def make_anno(name, value=None, match_regex=False, namespace=None):
    anno = dict()
    if name != None:
        anno['name'] = name
    if namespace == None:
        anno['qname'] = name
    else:
        anno['namespace'] = namespace
        anno['qname'] = namespace + ':' + name

    if value != None:
        anno['value'] = value

    if match_regex:
        anno['textMatching'] = 'REGEXP_EQUAL'
    else:
        anno['textMatching'] = 'EXACT_EQUAL'
    return anno


def _range_def(min_distance, max_distance):
    if min_distance == None and max_distance == None:
        return {'minDistance': 0, 'maxDistance': 0}
    else:
        return {'minDistance': min_distance if min_distance != None else 0, 'maxDistance': max_distance if max_distance != None else 0}


class Conjunction:
    def __init__(self):
        self.q = {'nodes': [], 'joins': []}

    def att(self, name, value=None, match_regex=False, namespace=None):
        """ Create a new attribute (node) search in the conjunction. The
        ID of the attribute is automatically created.
        This does not change the original query, but returns a new one.

        >>> from graphannis.query import Conjunction
        >>> q = Conjunction().att('pos', 'NN')
        """
        obj = copy.copy(self)

        node_anno = dict()

        if name == 'tok':
            node_anno['token'] = True
            node_anno.update(make_anno(None, value, match_regex, namespace))
        else:
            node_anno['token'] = False
            if name != 'node':
                node_anno.update(
                    make_anno(name, value, match_regex, namespace))

        node_idx = len(obj.q['nodes'])+1
        obj.q['nodes'].append(
            {node_idx: {
                'id': node_idx,
                'nodeAnnotations': [node_anno]
            }})
        return obj

    def precedence(self, n_left, n_right, min_distance=1, max_distance=1, seg_name=None):
        """Add a precedence (.) operator between to attributes.

        >>> q = Conjunction().att('pos', 'NN').att('tok', '').precedence('1', '2')
        """
        obj = copy.copy(self)

        op_def = {'op': 'Precedence',
                  'left': n_left, 'right': n_right
                  }
        op_def.update(_range_def(min_distance, max_distance))

        if seg_name != None:
            op_def['segmentation-name'] = seg_name

        obj.q['joins'].append(
            op_def
        )

        return obj

    def pointing(self, n_left, n_right, type_name, min_distance=1, max_distance=1, edge_anno=None):
        """Add a pointing (->type_name) operator between to attributes.

        >>> q = Conjunction().att('pos', 'NN').att('tok', '').pointing('1', '2', 'dep')
        """
        obj = copy.copy(self)

        op_def = {'op': 'Pointing',
                  'name': type_name,
                  'left': n_left, 'right': n_right
                  }
        op_def.update(_range_def(min_distance, max_distance))

        if edge_anno != None:
            op_def['edgeAnnotations'] = edge_anno

        obj.q['joins'].append(
            op_def
        )

        return obj

    def dominance(self, n_left, n_right, type_name=None, min_distance=1, max_distance=1, edge_anno=None):
        """Add a dominance (>) operator between to attributes.

        >>> q = Conjunction().att('pos', 'NN').att('tok', '').dominance('1', '2', make_anno('func', 'value'))
        """
        obj = copy.copy(self)

        op_def = {'op': 'Pointing',
                  'left': n_left, 'right': n_right
                  }
        if type_name != None:
            op_def['type'] = type_name

        op_def.update(_range_def(min_distance, max_distance))

        if edge_anno != None:
            op_def['edgeAnnotations'] = edge_anno

        obj.q['joins'].append(
            op_def
        )

        return obj

    def ident_cov(self, n_left, n_right):
        """Add a _=_ operator between to attributes.

        >>> q = Conjunction().att('pos', 'NN').att('tok', '').ident_cov('1', '2')
        """
        obj = copy.copy(self)

        op_def = {'op': 'IdenticalCoverage',
                  'left': n_left, 'right': n_right
                  }

        obj.q['joins'].append(
            op_def
        )

        return obj
