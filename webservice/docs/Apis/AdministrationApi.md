# AdministrationApi

All URIs are relative to *http://localhost:5711/v0*

Method | HTTP request | Description
------------- | ------------- | -------------
[**deleteGroup**](AdministrationApi.md#deleteGroup) | **DELETE** /groups/{name} | Delete the user group given by its name
[**exportPost**](AdministrationApi.md#exportPost) | **POST** /export | Get all requested corpora as ZIP-file
[**getJob**](AdministrationApi.md#getJob) | **GET** /jobs/{uuid} | Get the status of the background job with the UUID
[**importPost**](AdministrationApi.md#importPost) | **POST** /import | Import all corpora which are part of the uploaded ZIP-file
[**listGroups**](AdministrationApi.md#listGroups) | **GET** /groups | Get all available user groups
[**putGroup**](AdministrationApi.md#putGroup) | **PUT** /groups/{name} | Add or replace the user group given by its name


<a name="deleteGroup"></a>
# **deleteGroup**
> deleteGroup(name)

Delete the user group given by its name

### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **name** | **String**|  | [default to null]

### Return type

null (empty response body)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

<a name="exportPost"></a>
# **exportPost**
> inline_response_202 exportPost(inlineObject)

Get all requested corpora as ZIP-file

### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **inlineObject** | [**InlineObject**](..//Models/InlineObject.md)|  |

### Return type

[**inline_response_202**](..//Models/inline_response_202.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: application/json

<a name="getJob"></a>
# **getJob**
> getJob(uuid)

Get the status of the background job with the UUID

### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **uuid** | **String**|  | [default to null]

### Return type

null (empty response body)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

<a name="importPost"></a>
# **importPost**
> ImportResult importPost(body, overrideExisting)

Import all corpora which are part of the uploaded ZIP-file

    This will search for all GraphML and relANNIS files in the uploaded ZIP file and imports them.

### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **body** | **File**|  |
 **overrideExisting** | **Boolean**| If true, existing corpora will be overwritten by the uploaded ones. | [optional] [default to false]

### Return type

[**ImportResult**](..//Models/ImportResult.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/octet-stream
- **Accept**: application/json

<a name="listGroups"></a>
# **listGroups**
> List listGroups()

Get all available user groups

### Parameters
This endpoint does not need any parameter.

### Return type

[**List**](..//Models/Group.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

<a name="putGroup"></a>
# **putGroup**
> putGroup(name, group)

Add or replace the user group given by its name

### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **name** | **String**|  | [default to null]
 **group** | [**Group**](..//Models/Group.md)| The group to add |

### Return type

null (empty response body)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

