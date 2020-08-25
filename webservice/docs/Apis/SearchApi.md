# SearchApi

All URIs are relative to *http://localhost:5711/v0*

Method | HTTP request | Description
------------- | ------------- | -------------
[**count**](SearchApi.md#count) | **POST** /search/count | Count the number of results for a query.
[**find**](SearchApi.md#find) | **POST** /search/find | Find results for a query and return the IDs of the matched nodes.
[**frequency**](SearchApi.md#frequency) | **POST** /search/frequency | Find results for a query and return the IDs of the matched nodes.
[**nodeDescriptions**](SearchApi.md#nodeDescriptions) | **GET** /search/node-descriptions | Parses a query and returns a description for all the nodes in the query.
[**subgraphForQuery**](SearchApi.md#subgraphForQuery) | **GET** /corpora/{corpus}/subgraph-for-query | Get a subgraph of the corpus format given a list of nodes and a context.


<a name="count"></a>
# **count**
> CountExtra count(countQuery)

Count the number of results for a query.

### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **countQuery** | [**CountQuery**](..//Models/CountQuery.md)| The definition of the query to execute. |

### Return type

[**CountExtra**](..//Models/CountExtra.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

<a name="find"></a>
# **find**
> File find(findQuery)

Find results for a query and return the IDs of the matched nodes.

### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **findQuery** | [**FindQuery**](..//Models/FindQuery.md)| The definition of the query to execute. |

### Return type

[**File**](..//Models/file.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: text/plain, application/json

<a name="frequency"></a>
# **frequency**
> List frequency(frequencyQuery)

Find results for a query and return the IDs of the matched nodes.

### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **frequencyQuery** | [**FrequencyQuery**](..//Models/FrequencyQuery.md)| The definition of the query to execute. |

### Return type

[**List**](..//Models/FrequencyTableRow.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

<a name="nodeDescriptions"></a>
# **nodeDescriptions**
> List nodeDescriptions(query, queryLanguage)

Parses a query and returns a description for all the nodes in the query.

### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **query** | **String**| The query to parse | [optional] [default to null]
 **queryLanguage** | [**QueryLanguage**](..//Models/.md)|  | [optional] [default to null] [enum: AQL, AQLQuirksV3]

### Return type

[**List**](..//Models/QueryAttributeDescription.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

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

