import copy


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
        else:
            node_anno['token'] = False
            node_anno['name'] = name
            if namespace == None:
                node_anno['qname'] = name
            else:
                node_anno['namespace'] = namespace
                node_anno['qname'] = namespace + ':' + name

        if value != None:
            node_anno['value'] = value

        if match_regex:
            node_anno['textMatching'] = 'REGEXP_EQUAL'
        else:
            node_anno['textMatching'] = 'EXACT_EQUAL'



        node_idx = len(obj.q['nodes'])+1
        obj.q['nodes'].append(
            {node_idx: {
            'id': node_idx,
            'nodeAnnotations': [node_anno]
            }})
        return obj

    def precedence(self, n_left, n_right, min_distance=1, max_distance=1, seg_name=None):
        """Add a precedence operator between to attributes.

        >>> q = Conjunction().att('pos', 'NN').att('tok', '').precedence('1', '2')
        """
        obj = copy.copy(self)

        op_def = {'op' : 'Precedence',
        'minDistance' : max_distance, 'maxDistance' : max_distance,
        'left': n_left, 'right' : n_right
        }

        if seg_name != None:
            op_def['segmentation-name'] = seg_name

        obj.q['joins'].append(
            op_def
        )

        return obj
