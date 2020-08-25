# Documentation for graphANNIS

<a name="documentation-for-api-endpoints"></a>
## Documentation for API Endpoints

All URIs are relative to *http://localhost:5711/v0*

Class | Method | HTTP request | Description
------------ | ------------- | ------------- | -------------
*AdministrationApi* | [**deleteGroup**](Apis/AdministrationApi.md#deletegroup) | **DELETE** /groups/{name} | Delete the user group given by its name
*AdministrationApi* | [**exportPost**](Apis/AdministrationApi.md#exportpost) | **POST** /export | Get all requested corpora as ZIP-file
*AdministrationApi* | [**getJob**](Apis/AdministrationApi.md#getjob) | **GET** /jobs/{uuid} | Get the status of the background job with the UUID
*AdministrationApi* | [**importPost**](Apis/AdministrationApi.md#importpost) | **POST** /import | Import all corpora which are part of the uploaded ZIP-file
*AdministrationApi* | [**listGroups**](Apis/AdministrationApi.md#listgroups) | **GET** /groups | Get all available user groups
*AdministrationApi* | [**putGroup**](Apis/AdministrationApi.md#putgroup) | **PUT** /groups/{name} | Add or replace the user group given by its name
*AuthentificationApi* | [**localLogin**](Apis/AuthentificationApi.md#locallogin) | **POST** /local-login | Create JWT token for credentials of a locally configured account.
*CorporaApi* | [**components**](Apis/CorporaApi.md#components) | **GET** /corpora/{corpus}/components | List all edge components of the corpus.
*CorporaApi* | [**corpusConfiguration**](Apis/CorporaApi.md#corpusconfiguration) | **GET** /corpora/{corpus}/configuration | Get the corpus configuration object.
*CorporaApi* | [**deleteCorpus**](Apis/CorporaApi.md#deletecorpus) | **DELETE** /corpora/{corpus} | Delete the given corpus.
*CorporaApi* | [**edgeAnnotations**](Apis/CorporaApi.md#edgeannotations) | **GET** /corpora/{corpus}/edge-annotations/{type}/{layer}/{name}/ | List all annotations of the corpus for a given edge component
*CorporaApi* | [**getFile**](Apis/CorporaApi.md#getfile) | **GET** /corpora/{corpus}/files/{name} | Get an associated file for the corpus by its name.
*CorporaApi* | [**listCorpora**](Apis/CorporaApi.md#listcorpora) | **GET** /corpora | Get a list of all corpora the user is authorized to use.
*CorporaApi* | [**listFiles**](Apis/CorporaApi.md#listfiles) | **GET** /corpora/{corpus}/files | List the names of all associated file for the corpus.
*CorporaApi* | [**nodeAnnotations**](Apis/CorporaApi.md#nodeannotations) | **GET** /corpora/{corpus}/node-annotations | List all node annotations of the corpus.
*CorporaApi* | [**subgraphForNodes**](Apis/CorporaApi.md#subgraphfornodes) | **POST** /corpora/{corpus}/subgraph | Get a subgraph of the corpus format given a list of nodes and a context.
*CorporaApi* | [**subgraphForQuery**](Apis/CorporaApi.md#subgraphforquery) | **GET** /corpora/{corpus}/subgraph-for-query | Get a subgraph of the corpus format given a list of nodes and a context.
*SearchApi* | [**count**](Apis/SearchApi.md#count) | **POST** /search/count | Count the number of results for a query.
*SearchApi* | [**find**](Apis/SearchApi.md#find) | **POST** /search/find | Find results for a query and return the IDs of the matched nodes.
*SearchApi* | [**frequency**](Apis/SearchApi.md#frequency) | **POST** /search/frequency | Find results for a query and return the IDs of the matched nodes.
*SearchApi* | [**nodeDescriptions**](Apis/SearchApi.md#nodedescriptions) | **GET** /search/node-descriptions | Parses a query and returns a description for all the nodes in the query.
*SearchApi* | [**subgraphForQuery**](Apis/SearchApi.md#subgraphforquery) | **GET** /corpora/{corpus}/subgraph-for-query | Get a subgraph of the corpus format given a list of nodes and a context.


<a name="documentation-for-models"></a>
## Documentation for Models

 - [AnnoKey](.//Models/AnnoKey.md)
 - [Annotation](.//Models/Annotation.md)
 - [AnnotationComponentType](.//Models/AnnotationComponentType.md)
 - [Component](.//Models/Component.md)
 - [CorpusConfiguration](.//Models/CorpusConfiguration.md)
 - [CorpusConfigurationContext](.//Models/CorpusConfigurationContext.md)
 - [CorpusConfigurationView](.//Models/CorpusConfigurationView.md)
 - [CountExtra](.//Models/CountExtra.md)
 - [CountQuery](.//Models/CountQuery.md)
 - [ExampleQuery](.//Models/ExampleQuery.md)
 - [FindQuery](.//Models/FindQuery.md)
 - [FrequencyQuery](.//Models/FrequencyQuery.md)
 - [FrequencyQueryDefinition](.//Models/FrequencyQueryDefinition.md)
 - [FrequencyTableRow](.//Models/FrequencyTableRow.md)
 - [GraphAnnisError](.//Models/GraphAnnisError.md)
 - [GraphAnnisErrorAQLSyntaxError](.//Models/GraphAnnisErrorAQLSyntaxError.md)
 - [GraphAnnisErrorLoadingGraphFailed](.//Models/GraphAnnisErrorLoadingGraphFailed.md)
 - [Group](.//Models/Group.md)
 - [ImportResult](.//Models/ImportResult.md)
 - [InlineObject](.//Models/InlineObject.md)
 - [InlineObject1](.//Models/InlineObject1.md)
 - [InlineResponse202](.//Models/InlineResponse202.md)
 - [Job](.//Models/Job.md)
 - [LineColumn](.//Models/LineColumn.md)
 - [LineColumnRange](.//Models/LineColumnRange.md)
 - [QueryAttributeDescription](.//Models/QueryAttributeDescription.md)
 - [QueryLanguage](.//Models/QueryLanguage.md)
 - [SubgraphWithContext](.//Models/SubgraphWithContext.md)
 - [VisualizerRule](.//Models/VisualizerRule.md)


<a name="documentation-for-authorization"></a>
## Documentation for Authorization

<a name="bearerAuth"></a>
### bearerAuth

- **Type**: HTTP basic authentication

