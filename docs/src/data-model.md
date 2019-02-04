# Data model

The data model and concepts used by graphANNIS are similar to the ones of [Salt](http://corpus-tools.org/salt/) (see the [Salt model guide](https://github.com/korpling/salt/raw/master/gh-site/doc/salt_modelGuide.pdf) for more information).
Historically, Salt and ANNIS with its query language AQL have been developed in parallel, sharing concepts and ideas
how a linguistic corpus should be modeled as a directed labeled graph.
Still, there are differences because of the purpose of each model: Salt should represent the annotation and data sources without loosing
information while the ANNIS data model transforms the data model to allow an efficient search.

GraphANNIS uses a data model that allows performing searches with AQL (and thus is compatible with its data model).
By using graphs, as Salt does, it is more flexible in modeling the data and can be more close to Salt than the relational database scheme of older ANNIS version could be.
Some parts of the data model are exposed to the outside, e.g. when a user applies changes to a graph.
Others are internal and are used to index structures needed for AQL, but which can be deduced from the information in the public data model.
