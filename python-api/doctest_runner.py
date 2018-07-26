#!/usr/bin/env python3

if __name__ == '__main__':
    import doctest
    from graphannis import graph, cs, query
    import graphannis
    doctest.testmod(graph)
    doctest.testmod(cs)
    doctest.testmod(query)
