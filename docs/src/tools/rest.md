# REST service

GraphANNIS includes a tool to start a complete REST service that can be used to query and adminstrate corpora.
The [ANNIS web-frontend](https://github.com/korpling/ANNIS) uses this REST service for executing the AQL searches.
Using this REST service, it is also possible to implement a custom AQL web-interface e.g. for a specific corpus or analysis workflow with minimal effort.
In addition to [using graphANNIS as a library in you application](../embed.md), the REST API allows you to implement a web interface for a remote graphANNIS server.
