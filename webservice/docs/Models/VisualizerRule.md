# VisualizerRule
## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**element** | [**String**](string.md) | On which element type to trigger the visualizer on | [optional] [default to null]
**layer** | [**String**](string.md) | In which layer the element needs to be part of to trigger this visualizer.  Only relevant for edges, since only they are part of layers. If not given, elements of all layers trigger this visualization.  | [optional] [default to null]
**visUnderscoretype** | [**String**](string.md) | The abstract type of visualization, e.g. \&quot;tree\&quot;, \&quot;discourse\&quot;, \&quot;grid\&quot;, ... | [optional] [default to null]
**displayUnderscorename** | [**String**](string.md) | A text displayed to the user describing this visualization | [optional] [default to null]
**visibility** | [**String**](string.md) | The default display state of the visualizer before any user interaction. | [optional] [default to null]
**mappings** | [**Map**](string.md) | Additional configuration given as generic map of key values to the visualizer. | [optional] [default to null]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)

