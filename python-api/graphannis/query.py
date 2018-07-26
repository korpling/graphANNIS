import copy


class Conjunction:
    def __init__(self):
        self.q = {'nodes': [], 'joins': []}

    def att(self, name, value, match_regex=False, namespace=None):
        """ Create a new attribute (node) search in the conjunction. The
        ID of the attribute is automatically created.

        >>> from graphannis.query import Conjunction
        >>> q = Conjunction().att('pos', 'NN')
        """
        obj = copy.copy(self)

        node_anno = {
            'name': name,
            'value': value,
        }

        if namespace == None:
            node_anno['qname'] = name
        else:
            node_anno['namespace'] = namespace
            node_anno['qname'] = namespace + ':' + name

        if match_regex:
            node_anno['textMatching'] = 'REGEXP_EQUAL'
        else:
            node_anno['textMatching'] = 'EXACT_EQUAL'

        if name == 'tok':
            node_anno['token'] = True
        else:
            node_anno['token'] = False

        node_idx = len(obj.q['nodes'])+1
        obj.q['nodes'].append(
            {node_idx: {
            'id': node_idx,
            'nodeAnnotations': [node_anno]
            }})
        return obj
