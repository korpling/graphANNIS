# AuthentificationApi

All URIs are relative to *http://localhost:5711/v0*

Method | HTTP request | Description
------------- | ------------- | -------------
[**localLogin**](AuthentificationApi.md#localLogin) | **POST** /local-login | Create JWT token for credentials of a locally configured account.


<a name="localLogin"></a>
# **localLogin**
> String localLogin(inlineObject1)

Create JWT token for credentials of a locally configured account.

### Parameters

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **inlineObject1** | [**InlineObject1**](..//Models/InlineObject1.md)|  |

### Return type

[**String**](..//Models/string.md)

### Authorization

[bearerAuth](../README.md#bearerAuth)

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: text/plain

