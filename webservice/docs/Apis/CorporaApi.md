# CorporaApi

All URIs are relative to *http://localhost:5711/v0*

Method | HTTP request | Description
------------- | ------------- | -------------
[**components**](CorporaApi.md#components) | **GET** /corpora/{corpus}/components | List all edge components of the corpus.
[**corpusConfiguration**](CorporaApi.md#corpusConfiguration) | **GET** /corpora/{corpus}/configuration | Get the corpus configuration object.
[**deleteCorpus**](CorporaApi.md#deleteCorpus) | **DELETE** /corpora/{corpus} | Delete the given corpus.
[**edgeAnnotations**](CorporaApi.md#edgeAnnotations) | **GET** /corpora/{corpus}/edge-annotations/{type}/{layer}/{name}/ | List all annotations of the corpus for a given edge component
[**getFile**](CorporaApi.md#getFile) | **GET** /corpora/{corpus}/files/{name} | Get an associated file for the corpus by its name.
[**listCorpora**](CorporaApi.md#listCorpora) | **GET** /corpora | Get a list of all corpora the user is authorized to use.
[**listFiles**](CorporaApi.md#listFiles) | **GET** /corpora/{corpus}/files | List the names of all associated file for the corpus.
[**nodeAnnotations**](CorporaApi.md#nodeAnnotations) | **GET** /corpora/{corpus}/node-annotations | List all node annotations of the corpus.
[**subgraphForNodes**](CorporaApi.md#subgraphForNodes) | **POST** /corpora/{corpus}/subgraph | Get a subgraph of the corpus format given a list of nodes and a context.
[**subgraphForQuery**](CorporaApi.md#subgraphForQuery) | **GET** /corpora/{corpus}/subgraph-for-query | Get a subgraph of the corpus format given a list of nodes and a context.


<a name="components"></a>
# **components**
> List components(corpus, type, name)

List all edge components of the corpus.

### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **corpus** | **String**| The name of the corpus to get the components for. | [default to null]
 **type** | **String**| Only return components with this type. | [optional] [default to null]
 **name** | **String**| Only return components with this name. | [optional] [default to null]

### Return type

[**List**](..//Models/Component.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

<a name="corpusConfiguration"></a>
# **corpusConfiguration**
> CorpusConfiguration corpusConfiguration(corpus)

Get the corpus configuration object.

    The corpus configuration is created by the corpus authors to configure how the corpus should be displayed in query engines and visualizers.

### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **corpus** | **String**| The name of the corpus to get the configuration for. | [default to null]

### Return type

[**CorpusConfiguration**](..//Models/CorpusConfiguration.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

<a name="deleteCorpus"></a>
# **deleteCorpus**
> deleteCorpus(corpus)

Delete the given corpus.

### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **corpus** | **String**|  | [default to null]

### Return type

null (empty response body)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

<a name="edgeAnnotations"></a>
# **edgeAnnotations**
> List edgeAnnotations(corpus, type, layer, name, listValues, onlyMostFrequentValues)

List all annotations of the corpus for a given edge component

### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **corpus** | **String**| The name of the corpus to get the configuration for. | [default to null]
 **type** | **String**| The component type. | [default to null]
 **layer** | **String**| The component layer. | [default to null]
 **name** | **String**| The component name. | [default to null]
 **listValues** | **Boolean**| If true, possible values are returned. | [optional] [default to false]
 **onlyMostFrequentValues** | **Boolean**| If true, only the most frequent value per annotation is returned. | [optional] [default to false]

### Return type

[**List**](..//Models/Annotation.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

<a name="getFile"></a>
# **getFile**
> File getFile(corpus, name)

Get an associated file for the corpus by its name.

    The annotation graph of a corpus can contain special nodes of the type \&quot;file\&quot;,  which are connected to (sub-) corpus and document nodes with a &#x60;PartOf&#x60; relation. This endpoint allows to access the content of these file nodes. It supports [HTTP range requests](https://developer.mozilla.org/en-US/docs/Web/HTTP/Range_requests)  if you only need to access parts of the file. 

### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **corpus** | **String**| The name of the corpus to get the configuration for. | [default to null]
 **name** | **String**| The name of the file node. | [default to null]

### Return type

[**File**](..//Models/file.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: default

<a name="listCorpora"></a>
# **listCorpora**
> List listCorpora()

Get a list of all corpora the user is authorized to use.

### Parameters
This endpoint does not need any parameter.

### Return type

[**List**](..//Models/string.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

<a name="listFiles"></a>
# **listFiles**
> List listFiles(corpus, node)

List the names of all associated file for the corpus.

### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **corpus** | **String**| The name of the corpus to get the configuration for. | [default to null]
 **node** | **String**| If given, only the files for the (sub-) corpus or document with this ID are returned. | [optional] [default to null]

### Return type

[**List**](..//Models/string.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: default

<a name="nodeAnnotations"></a>
# **nodeAnnotations**
> List nodeAnnotations(corpus, listValues, onlyMostFrequentValues)

List all node annotations of the corpus.

### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **corpus** | **String**| The name of the corpus to get the configuration for. | [default to null]
 **listValues** | **Boolean**| If true, possible values are returned. | [optional] [default to false]
 **onlyMostFrequentValues** | **Boolean**| If true, only the most frequent value per annotation is returned. | [optional] [default to false]

### Return type

[**List**](..//Models/Annotation.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

<a name="subgraphForNodes"></a>
# **subgraphForNodes**
> File subgraphForNodes(corpus, subgraphWithContext)

Get a subgraph of the corpus format given a list of nodes and a context.

    This creates a subgraph for node IDs, which can e.g. generated by executing a &#x60;find&#x60; query. The subgraph contains  - the given nodes,  - all tokens that are covered by the given nodes, - all tokens left and right in the given context from the tokens covered by the give nodes, - all other nodes covering the tokens of the given context. The annotation graph also includes all edges between the included nodes. 

### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **corpus** | **String**| The name of the corpus to get the subgraph for. | [default to null]
 **subgraphWithContext** | [**SubgraphWithContext**](..//Models/SubgraphWithContext.md)| The definition of the subgraph to extract. |

### Return type

[**File**](..//Models/file.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/xml

<a name="subgraphForQuery"></a>
# **subgraphForQuery**
> File subgraphForQuery(corpus, query, queryLanguage, componentTypeFilter)

Get a subgraph of the corpus format given a list of nodes and a context.

    This only includes the nodes that are the result of the given query and no context is created automatically. The annotation graph also includes all edges between the included nodes. 

### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **corpus** | **String**| The name of the corpus to get the subgraph for. | [default to null]
 **query** | **String**| The query which defines the nodes to include. | [default to null]
 **queryLanguage** | [**QueryLanguage**](..//Models/.md)|  | [optional] [default to null] [enum: AQL, AQLQuirksV3]
 **componentTypeFilter** | [**AnnotationComponentType**](..//Models/.md)| If given, restricts the included edges to components with the given type. | [optional] [default to null] [enum: Coverage, Dominance, Pointing, Ordering, LeftToken, RightToken, PartOf]

### Return type

[**File**](..//Models/file.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/xml

