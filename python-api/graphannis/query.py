import copy

class Conjunction:
    def __init__(self):
        self.q = {'nodes' : [], 'joins': []}


    def anno(self, name, value):
        obj = copy.copy(self)

        node_idx = len(obj.q['nodes'])+1
        obj.q['nodes'].append({'id' : node_idx})
        return obj
