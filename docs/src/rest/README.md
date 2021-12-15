# REST service

GraphANNIS includes a tool to start a complete REST service that can be used to query and administrate corpora.
The [ANNIS web-frontend](https://github.com/korpling/ANNIS) uses this REST service for executing the AQL searches.
Using this REST service, it is also possible to implement a custom AQL web-interface e.g. for a specific corpus or analysis workflow with minimal effort.
In addition to [using graphANNIS as a library in you application](../embed.md), the REST API allows you to implement a web interface for a remote graphANNIS server.

You can just execute the `graphannis-webservice` executable[^rename] to start a web-server with default settings and on port 5711 which will listen to requests from `localhost`.
SSL is not supported, so if you want to make the service accessible from the outside you should use a proxy server with encryption enabled and a valid certificate.

The graphANNIS REST API is specified and documented in [OpenAPI 3](https://swagger.io/docs/specification/about/).
The specification file can also be used to auto-generate client code, e.g. with the [OpenAPI Generator](https://github.com/OpenAPITools/openapi-generator#overview).
The documentation can be displayed with any OpenAPI 3 viewer using the URL to the [released openapi.yml file](https://raw.githubusercontent.com/korpling/graphANNIS/main/webservice/src/openapi.yml).
We also include the `webservice/api-docs.html` file in our repository which includes an interactive rendering of the documentation.

[^rename]: When downloading a binary from the release page, on MacOS you might need to rename the downloaded file from `graphannis-webservice.osx` to `graphannis-webservice`. The executable is called `graphannis-webservice.exe` on Windows.
