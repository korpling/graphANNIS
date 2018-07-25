import networkx as nx
import sys, re

def salt_uri_from_match(match):
    elements = re.split('::', match, 3)
    if len(elements) == 3:
        return elements[2]
    elif len(elements) == 2:
        return elements[1]
    else:
        return elements[0]

def matches(G):
    matchList = list()
    if G:
        for name, n in G.nodes_iter(data=True):
            if "annis::matchednode" in n:
                matchList.append((name, n["annis::matchednode"]))
    return list(map(lambda v: v[0], sorted(matchList, key=lambda v: v[1])))

def isCoveragePath(G, path):
    # check that each edge is not a pointing relation
    for i in range(len(path)-1):
        edgeType = G.edge[path[i]][path[i+1]]["salt::type"]
        if (edgeType != "SDOMINANCE_RELATION" and edgeType != "SSPANNING_RELATION"
                    and edgeType != "STEXTUAL_RELATION"):
            return False
    return True

def textNodeForToken(G, t):
    for outedge in G.out_edges(nbunch=t, data="salt::type"):
        if outedge[2] == "STEXTUAL_RELATION":
            return outedge[1]
    return None

def textForToken(G, tokenSet):

    text = ""
    textNode = None
    start = 0
    end = 0

    for t in tokenSet:
        ds_id = textNodeForToken(G, t)
        if ds_id:
            if textNode:
            # check that the token belongs to the same text node
                if textNode != ds_id:
                    return None
            else:
                textNode = ds_id
                text = G.node[textNode]["salt::SDATA"]
                start = len(text)
            # update minimal and maximal start/end values
            start = min(start, int(G.edge[t][textNode]["salt::SSTART"]))
            end = max(end, int(G.edge[t][textNode]["salt::SEND"]))
    return text[start:end]

def coveredToken(G, source):
    tokens = set()
    # find all sequential datasources
    for n_id, n in G.nodes_iter(data=True):
        if n["salt::type"] == "STEXTUAL_DS":
            ds_id = n_id
            # find all coverage paths between the source and the data source
            paths = nx.all_simple_paths(G, source, ds_id)
            for p in paths:
                if isCoveragePath(G, p):
                    # the second last element in the path must be the token
                    tokens.add(p[len(p)-2])
    return tokens

def sortByText(G, nodes):
    nodeByStart = list()

    for n in nodes:
        # get the covered token
        covtokens = coveredToken(G, n)
        minStart = sys.maxsize
        for t in covtokens:
            ds_id = textNodeForToken(G, t)
            if ds_id:
                minStart = min(minStart, int(G.edge[t][ds_id]["salt::SSTART"]))
                                # add the token together withs its "salt::SSTART" attribute to the list
        nodeByStart.append([n, minStart])
    # first sort the list by the second value of each list item (the start value),
    # then only use the first value for the sorted list (map function)
    return list(map(lambda v: v[0], sorted(nodeByStart, key=lambda v: v[1])))

def __dotAttributesForNodes(G):
    for n_id, n in G.nodes(data=True):
        lbl = ""
        existingAtts=list()
        for attName in n:
            lbl = lbl + attName + "=" + str(n[attName]) + "\n"
            existingAtts.append(attName)
        n["label"] = lbl
        n["shape"] = "box"
        # remove all other labels
        for a in existingAtts:
            del n[a]
            
def __dotAttributesForEdges(G):
    for e_source, e_target, e in G.edges(data=True):
        lbl = ""
        existingAtts=list()
        for attName in e:
            if attName != "id":
                lbl = lbl + attName + "=" + str(e[attName]) + "\n"
            existingAtts.append(attName)
        e["label"] = lbl
        # remove all other labels
        for a in existingAtts:
            del e[a]

def toDot(G, filename):
    C = G.copy()
    # add attribute "label" for nodes and edges
    __dotAttributesForNodes(C)
    __dotAttributesForEdges(C)

    nx.drawing.nx_pydot.write_dot(C, filename)
