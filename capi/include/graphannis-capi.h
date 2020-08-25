/* SPDX-License-Identifier:  Apache-2.0
Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.s 
*/

#ifndef graphannis_capi_h
#define graphannis_capi_h

/* Generated with cbindgen:0.13.2 */

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * Specifies the type of component of the annotation graph. The types of this enum carray certain semantics about the edges of the graph components their are used in.
 */
typedef enum {
  /**
   * Edges between a span node and its tokens. Implies text coverage.
   */
  Coverage,
  /**
   * Edges between a structural node and any other structural node, span or token. Implies text coverage.
   */
  Dominance = 2,
  /**
   * Edge between any node.
   */
  Pointing,
  /**
   * Edge between two tokens implying that the source node comes before the target node in the textflow.
   */
  Ordering,
  /**
   * Explicit edge between any non-token node and the left-most token it covers.
   */
  LeftToken,
  /**
   * Explicit edge between any non-token node and the right-most token it covers.
   */
  RightToken,
  /**
   * Implies that the source node belongs to the parent corpus/subcorpus/document/datasource node.
   */
  PartOf,
} AnnisAnnotationComponentType;

/**
 * An enum of all supported input formats of graphANNIS.
 */
typedef enum {
  /**
   * Legacy [relANNIS import file format](http://korpling.github.io/ANNIS/4.0/developer-guide/annisimportformat.html)
   */
  RelANNIS,
  /**
   * [GraphML](http://graphml.graphdrawing.org/) based export-format, suitable to be imported from other graph databases.
   * This format follows the extensions/conventions of the Neo4j [GraphML module](https://neo4j.com/docs/labs/apoc/current/import/graphml/).
   */
  GraphML,
} AnnisImportFormat;

/**
 * Different levels of logging. Higher levels activate logging of events of lower levels as well.
 */
typedef enum {
  Off,
  Error,
  Warn,
  Info,
  Debug,
  Trace,
} AnnisLogLevel;

/**
 * An enum over all supported query languages of graphANNIS.
 *
 * Currently, only the ANNIS Query Language (AQL) and its variants are supported, but this enum allows us to add a support for older query language versions
 * or completely new query languages.
 */
typedef enum {
  AQL,
  /**
   * Emulates the (sometimes problematic) behavior of AQL used in ANNIS 3
   */
  AQLQuirksV3,
} AnnisQueryLanguage;

/**
 * Defines the order of results of a `find` query.
 */
typedef enum {
  /**
   * Order results by their document name and the the text position of the match.
   */
  Normal,
  /**
   * Inverted the order of `Normal`.
   */
  Inverted,
  /**
   * A random ordering which is **not stable**. Each new query will result in a different order.
   */
  Randomized,
  /**
   * Results are not ordered at all, but also not actively randomized
   * Each new query *might* result in a different order.
   */
  NotSorted,
} AnnisResultOrder;

/**
 * An annotation with a qualified name and a value.
 */
typedef struct AnnisAnnotation AnnisAnnotation;

/**
 * Identifies an edge component of the graph.
 */
typedef struct AnnisComponent_AnnotationComponentType AnnisComponent_AnnotationComponentType;

/**
 * A thread-safe API for managing corpora stored in a common location on the file system.
 *
 * Multiple corpora can be part of a corpus storage and they are identified by their unique name.
 * Corpora are loaded from disk into main memory on demand:
 * An internal main memory cache is used to avoid re-loading a recently queried corpus from disk again.
 */
typedef struct AnnisCorpusStorage AnnisCorpusStorage;

typedef struct AnnisDiskMap_u64__UpdateEvent AnnisDiskMap_u64__UpdateEvent;

/**
 * A representation of a graph including node annotations and edges.
 * Edges are partioned into components and each component is implemented by specialized graph storage implementation.
 *
 * Graphs can have an optional location on the disk.
 * In this case, changes to the graph via the [apply_update(...)](#method.apply_update) function are automatically persisted to this location.
 *
 */
typedef struct AnnisGraph_AnnotationComponentType AnnisGraph_AnnotationComponentType;

typedef struct AnnisIterPtr_NodeID AnnisIterPtr_NodeID;

typedef struct AnnisVec_Annotation AnnisVec_Annotation;

typedef struct AnnisVec_AnnotationComponent AnnisVec_AnnotationComponent;

typedef struct AnnisVec_CString AnnisVec_CString;

typedef struct AnnisVec_Edge AnnisVec_Edge;

typedef struct AnnisVec_Error AnnisVec_Error;

typedef struct AnnisVec_FrequencyTableRow_CString AnnisVec_FrequencyTableRow_CString;

typedef struct AnnisVec_QueryAttributeDescription AnnisVec_QueryAttributeDescription;

typedef struct AnnisVec_Vec_CString AnnisVec_Vec_CString;

typedef AnnisComponent_AnnotationComponentType AnnisAnnotationComponent;

/**
 * A list of changes to apply to an graph.
 */
typedef struct {
  AnnisDiskMap_u64__UpdateEvent diffs;
  uint64_t event_counter;
} AnnisGraphUpdate;

/**
 * A list of multiple errors.
 */
typedef AnnisVec_Error AnnisErrorList;

/**
 * A specialization of the [`Graph`](struct.Graph.html), using components needed to represent and query corpus annotation graphs.
 */
typedef AnnisGraph_AnnotationComponentType AnnisAnnotationGraph;

/**
 * A struct that contains the extended results of the count query.
 */
typedef struct {
  /**
   * Total number of matches.
   */
  uint64_t match_count;
  /**
   * Number of documents with at least one match.
   */
  uint64_t document_count;
} AnnisCountExtra;

/**
 * Definition of the result of a `frequency` query.
 */
typedef AnnisVec_FrequencyTableRow_CString AnnisFrequencyTable_CString;

/**
 * Simple definition of a matrix from a single data type.
 */
typedef AnnisVec_Vec_CString AnnisMatrix_CString;

/**
 * Unique internal identifier for a single node.
 */
typedef uint64_t AnnisNodeID;

/**
 * Directed edge between a source and target node which are identified by their ID.
 */
typedef struct {
  AnnisNodeID source;
  AnnisNodeID target;
} AnnisEdge;

/**
 * Get the name of the given annotation object.
 */
char *annis_annotation_name(const AnnisAnnotation *ptr);

/**
 * Get the namespace of the given annotation object.
 */
char *annis_annotation_ns(const AnnisAnnotation *ptr);

/**
 * Get the value of the given annotation object.
 */
char *annis_annotation_val(const AnnisAnnotation *ptr);

/**
 * Get the layer of the given component.
 *
 * The returned string must be deallocated by the caller using annis_str_free()!
 */
char *annis_component_layer(const AnnisAnnotationComponent *c);

/**
 * Get the name of the given component.
 *
 * The returned string must be deallocated by the caller using annis_str_free()!
 */
char *annis_component_name(const AnnisAnnotationComponent *c);

/**
 * Get the type of the given component.
 */
AnnisAnnotationComponentType annis_component_type(const AnnisAnnotationComponent *c);

/**
 * Apply a sequence of updates (`update` parameter) to this graph for a corpus given by the `corpus_name` parameter.
 *
 * - `ptr` - The corpus storage object.
 * - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
 *
 * It is ensured that the update process is atomic and that the changes are persisted to disk if the error list is empty.
 */
void annis_cs_apply_update(AnnisCorpusStorage *ptr,
                           const char *corpus_name,
                           AnnisGraphUpdate *update,
                           AnnisErrorList **err);

/**
 * Return the copy of the graph of the corpus structure given by `corpus_name`.
 *
 * - `ptr` - The corpus storage object.
 * - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
 */
AnnisAnnotationGraph *annis_cs_corpus_graph(const AnnisCorpusStorage *ptr,
                                            const char *corpus_name,
                                            AnnisErrorList **err);

/**
 * Count the number of results for a `query`.
 * - `ptr` - The corpus storage object.
 * - `corpus_names` - The name of the corpora to execute the query on.
 * - `query` - The query as string.
 * - `query_language` The query language of the query (e.g. AQL).
 * - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
 *
 * Returns the count as number.
 */
uint64_t annis_cs_count(const AnnisCorpusStorage *ptr,
                        const AnnisVec_CString *corpus_names,
                        const char *query,
                        AnnisQueryLanguage query_language,
                        AnnisErrorList **err);

/**
 * Count the number of results for a `query` and return both the total number of matches and also the number of documents in the result set.
 *
 * - `ptr` - The corpus storage object.
 * - `corpus_names` - The name of the corpora to execute the query on.
 * - `query` - The query as string.
 * - `query_language` The query language of the query (e.g. AQL).
 * - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
 */
AnnisCountExtra annis_cs_count_extra(const AnnisCorpusStorage *ptr,
                                     const AnnisVec_CString *corpus_names,
                                     const char *query,
                                     AnnisQueryLanguage query_language,
                                     AnnisErrorList **err);

/**
 * Delete a corpus from this corpus storage.
 * Returns `true` if the corpus was successfully deleted and `false` if no such corpus existed.
 *
 * - `ptr` - The corpus storage object.
 * - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
 */
bool annis_cs_delete(AnnisCorpusStorage *ptr, const char *corpus, AnnisErrorList **err);

/**
 * Find all results for a `query` and return the match ID for each result.
 *
 * The query is paginated and an offset and limit can be specified.
 *
 * - `ptr` - The corpus storage object.
 * - `corpus_names` - The name of the corpora to execute the query on.
 * - `query` - The query as string.
 * - `query_language` The query language of the query (e.g. AQL).
 * - `offset` - Skip the `n` first results, where `n` is the offset.
 * - `limit` - Return at most `n` matches, where `n` is the limit.  Use `None` to allow unlimited result sizes.
 * - `order` - Specify the order of the matches.
 * - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
 *
 * Returns a vector of match IDs, where each match ID consists of the matched node annotation identifiers separated by spaces.
 * You can use the `annis_cs_subgraph(...)` method to get the subgraph for a single match described by the node annnotation identifiers.
 */
AnnisVec_CString *annis_cs_find(const AnnisCorpusStorage *ptr,
                                const AnnisVec_CString *corpus_names,
                                const char *query,
                                AnnisQueryLanguage query_language,
                                size_t offset,
                                const size_t *limit,
                                AnnisResultOrder order,
                                AnnisErrorList **err);

/**
 * Frees the reference to the corpus storage object.
 * - `ptr` - The corpus storage object.
 */
void annis_cs_free(AnnisCorpusStorage *ptr);

/**
 * Execute a frequency query.
 *
 * - `ptr` - The corpus storage object.
 * - `corpus_names` - The name of the corpora to execute the query on.
 * - `query` - The query as string.
 * - `query_language` The query language of the query (e.g. AQL).
 * - `frequency_query_definition` - A string representation of the list of frequency query definitions.
 * - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
 *
 * Returns a frequency table of strings.
 */
AnnisFrequencyTable_CString *annis_cs_frequency(const AnnisCorpusStorage *ptr,
                                                const AnnisVec_CString *corpus_names,
                                                const char *query,
                                                AnnisQueryLanguage query_language,
                                                const char *frequency_query_definition,
                                                AnnisErrorList **err);

/**
 * Import a corpus from an external location on the file system into this corpus storage.
 *
 * - `ptr` - The corpus storage object.
 * - `path` - The location on the file system where the corpus data is located.
 * - `format` - The format in which this corpus data is stored.
 * - `corpus_name` - Optionally override the name of the new corpus for file formats that already provide a corpus name.
 * - `disk_based` - If `true`, prefer disk-based annotation and graph storages instead of memory-only ones.
 * - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
 *
 * Returns the name of the imported corpus.
 * The returned string must be deallocated by the caller using annis_str_free()!
 */
char *annis_cs_import_from_fs(AnnisCorpusStorage *ptr,
                              const char *path,
                              AnnisImportFormat format,
                              const char *corpus_name,
                              bool disk_based,
                              bool overwrite_existing,
                              AnnisErrorList **err);

/**
 * List all available corpora in the corpus storage.
 *
 * - `ptr` - The corpus storage object.
 * - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
 */
AnnisVec_CString *annis_cs_list(const AnnisCorpusStorage *ptr, AnnisErrorList **err);

/**
 * Returns a list of all components of a corpus given by `corpus_name` and the component type.
 *
 * - `ptr` - The corpus storage object.
 * - `ctype` -Filter by the component type.
 */
AnnisVec_AnnotationComponent *annis_cs_list_components_by_type(AnnisCorpusStorage *ptr,
                                                               const char *corpus_name,
                                                               AnnisAnnotationComponentType ctype);

/**
 * Returns a list of all edge annotations of a corpus given by `corpus_name` and the component.
 *
 * - `ptr` - The corpus storage object.
 * - `list_values` - If true include the possible values in the result.
 * - `component_type` - The type of the edge component.
 * - `component_name` - The name of the edge component.
 * - `component_layer` - The layer of the edge component.
 * - `only_most_frequent_values` - If both this argument and `list_values` are true, only return the most frequent value for each annotation name.
 * - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
 */
AnnisMatrix_CString *annis_cs_list_edge_annotations(const AnnisCorpusStorage *ptr,
                                                    const char *corpus_name,
                                                    AnnisAnnotationComponentType component_type,
                                                    const char *component_name,
                                                    const char *component_layer,
                                                    bool list_values,
                                                    bool only_most_frequent_values);

/**
 * Returns a list of all node annotations of a corpus given by `corpus_name`.
 *
 * - `ptr` - The corpus storage object.
 * - `list_values` - If true include the possible values in the result.
 * - `only_most_frequent_values` - If both this argument and `list_values` are true, only return the most frequent value for each annotation name.
 * - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
 */
AnnisMatrix_CString *annis_cs_list_node_annotations(const AnnisCorpusStorage *ptr,
                                                    const char *corpus_name,
                                                    bool list_values,
                                                    bool only_most_frequent_values);

/**
 * Parses a `query`and return a list of descriptions for its nodes.
 *
 * - `ptr` - The corpus storage object.
 * - `query` - The query to be analyzed.
 * - `query_language` - The query language of the query (e.g. AQL).
 * - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
 */
AnnisVec_QueryAttributeDescription *annis_cs_node_descriptions(const AnnisCorpusStorage *ptr,
                                                               const char *query,
                                                               AnnisQueryLanguage query_language,
                                                               AnnisErrorList **err);

/**
 * Return the copy of a subgraph which includes all nodes that belong to any of the given list of sub-corpus/document identifiers.
 *
 * - `ptr` - The corpus storage object.
 * - `corpus_name` - The name of the corpus for which the subgraph should be generated from.
 * - `corpus_ids` - A set of sub-corpus/document identifiers describing the subgraph.
 * - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
 */
AnnisAnnotationGraph *annis_cs_subcorpus_graph(const AnnisCorpusStorage *ptr,
                                               const char *corpus_name,
                                               const AnnisVec_CString *corpus_ids,
                                               AnnisErrorList **err);

/**
 * Return the copy of a subgraph which includes the given list of node annotation identifiers,
 * the nodes that cover the same token as the given nodes and
 * all nodes that cover the token which are part of the defined context.
 *
 * - `ptr` - The corpus storage object.
 * - `corpus_name` - The name of the corpus for which the subgraph should be generated from.
 * - `node_ids` - A set of node annotation identifiers describing the subgraph.
 * - `ctx_left` and `ctx_right` - Left and right context in token distance to be included in the subgraph.
 * - `segmentation` - The name of the segmentation which should be used to as base for the context. Use `None` to define the context in the default token layer.
 * - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
 */
AnnisAnnotationGraph *annis_cs_subgraph(const AnnisCorpusStorage *ptr,
                                        const char *corpus_name,
                                        const AnnisVec_CString *node_ids,
                                        size_t ctx_left,
                                        size_t ctx_right,
                                        const char *segmentation,
                                        AnnisErrorList **err);

/**
 * Return the copy of a subgraph which includes all nodes matched by the given `query`.
 *
 * - `ptr` - The corpus storage object.
 * - `corpus_name` - The name of the corpus for which the subgraph should be generated from.
 * - `query` - The query which defines included nodes.
 * - `query_language` - The query language of the query (e.g. AQL).
 * - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
 */
AnnisAnnotationGraph *annis_cs_subgraph_for_query(const AnnisCorpusStorage *ptr,
                                                  const char *corpus_name,
                                                  const char *query,
                                                  AnnisQueryLanguage query_language,
                                                  AnnisErrorList **err);

/**
 * Return the copy of a subgraph which includes all nodes matched by the given `query` and an additional filter.
 *
 * - `ptr` - The corpus storage object.
 * - `corpus_name` - The name of the corpus for which the subgraph should be generated from.
 * - `query` - The query which defines included nodes.
 * - `query_language` - The query language of the query (e.g. AQL).
 * - `component_type_filter` - Only include edges of that belong to a component of the given type.
 * - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
 */
AnnisAnnotationGraph *annis_cs_subgraph_for_query_with_ctype(const AnnisCorpusStorage *ptr,
                                                             const char *corpus_name,
                                                             const char *query,
                                                             AnnisQueryLanguage query_language,
                                                             AnnisAnnotationComponentType component_type_filter,
                                                             AnnisErrorList **err);

/**
 * Unloads a corpus from the cache.
 */
void annis_cs_unload(AnnisCorpusStorage *ptr, const char *corpus);

/**
 * Parses a `query` and checks if it is valid.
 *
 * - `ptr` - The corpus storage object.
 * - `corpus_names` - The name of the corpora the query would be executed on (needed to catch certain corpus-specific semantic errors).
 * - `query` - The query as string.
 * - `query_language` The query language of the query (e.g. AQL).
 * - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
 *
 * Returns `true` if valid and an error with the parser message if invalid.
 */
bool annis_cs_validate_query(const AnnisCorpusStorage *ptr,
                             const AnnisVec_CString *corpus_names,
                             const char *query,
                             AnnisQueryLanguage query_language,
                             AnnisErrorList **err);

/**
 * Create a new instance with a an automatic determined size of the internal corpus cache.
 *
 * Currently, set the maximum cache size to 25% of the available/free memory at construction time.
 * This behavior can change in the future.
 *
 * - `db_dir` - The path on the filesystem where the corpus storage content is located. Must be an existing directory.
 * - `use_parallel_joins` - If `true` parallel joins are used by the system, using all available cores.
 * - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
 */
AnnisCorpusStorage *annis_cs_with_auto_cache_size(const char *db_dir,
                                                  bool use_parallel_joins,
                                                  AnnisErrorList **err);

/**
 * Create a new corpus storage with an manually defined maximum cache size.
 *
 * - `db_dir` - The path on the filesystem where the corpus storage content is located. Must be an existing directory.
 * - `max_cache_size` - Fixed maximum size of the cache in bytes.
 * - `use_parallel_joins` - If `true` parallel joins are used by the system, using all available cores.
 * - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
 */
AnnisCorpusStorage *annis_cs_with_max_cache_size(const char *db_dir,
                                                 uintptr_t max_cache_size,
                                                 bool use_parallel_joins,
                                                 AnnisErrorList **err);

/**
 * Get the kind or type for the error at position `i` in the list.
 */
const char *annis_error_get_kind(const AnnisErrorList *ptr, size_t i);

/**
 * Get the message for the error at position `i` in the list.
 */
const char *annis_error_get_msg(const AnnisErrorList *ptr, size_t i);

/**
 * Returns the number of errors in the list.
 */
size_t annis_error_size(const AnnisErrorList *ptr);

/**
 * Frees the internal object given as `ptr` argument.
 */
void annis_free(void *ptr);

/**
 * Get the count of the `row` of the frequency table.
 */
size_t annis_freqtable_str_count(const AnnisFrequencyTable_CString *ptr, size_t row);

/**
 * Get a read-only reference to the string at the at position (`row`, `col`) of the frequency table.
 */
const char *annis_freqtable_str_get(const AnnisFrequencyTable_CString *ptr, size_t row, size_t col);

/**
 * Returns the number of columns of the frequency table.
 */
size_t annis_freqtable_str_ncols(const AnnisFrequencyTable_CString *ptr);

/**
 * Returns the number of rows of the frequency table.
 */
size_t annis_freqtable_str_nrows(const AnnisFrequencyTable_CString *ptr);

/**
 * Return a vector of all components for the graph `g`.
 */
AnnisVec_AnnotationComponent *annis_graph_all_components(const AnnisAnnotationGraph *g);

/**
 * Return a vector of all components for the graph `g` and the given component type.
 */
AnnisVec_AnnotationComponent *annis_graph_all_components_by_type(const AnnisAnnotationGraph *g,
                                                                 AnnisAnnotationComponentType ctype);

/**
 * Return a vector of annnotations for the given `edge` in the `component` of graph `g.
 */
AnnisVec_Annotation *annis_graph_annotations_for_edge(const AnnisAnnotationGraph *g,
                                                      AnnisEdge edge,
                                                      const AnnisAnnotationComponent *component);

/**
 * Return a vector of all annotations for the given `node` in the graph `g`.
 */
AnnisVec_Annotation *annis_graph_annotations_for_node(const AnnisAnnotationGraph *g,
                                                      AnnisNodeID node);

/**
 * Return an iterator over all nodes of the graph `g` and the given `node_type` (e.g. "node" or "corpus").
 */
AnnisIterPtr_NodeID *annis_graph_nodes_by_type(const AnnisAnnotationGraph *g,
                                               const char *node_type);

/**
 * Return a vector of all outgoing edges for the graph `g`, the `source` node and the given `component`.
 */
AnnisVec_Edge *annis_graph_outgoing_edges(const AnnisAnnotationGraph *g,
                                          AnnisNodeID source,
                                          const AnnisAnnotationComponent *component);

/**
 * Add "add edge" action to the graph update object.
 *
 * - `ptr` - The graph update object.
 * - `source_node` - Name of source node of the new edge.
 * - `target_node` - Name of target node of the new edge.
 * - `layer` - Layer of the new edge.
 * - `component_type` - Type of the component of the new edge.
 * - `component_name` - Name of the component of the new edge.
 * - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
 */
void annis_graphupdate_add_edge(AnnisGraphUpdate *ptr,
                                const char *source_node,
                                const char *target_node,
                                const char *layer,
                                const char *component_type,
                                const char *component_name,
                                AnnisErrorList **err);

/**
 * Add "add edge label" action to the graph update object.
 *
 * - `ptr` - The graph update object.
 * - `source_node` - Name of source node of the edge.
 * - `target_node` - Name of target node of the edge.
 * - `layer` - Layer of the edge.
 * - `component_type` - Type of the component of the edge.
 * - `component_name` - Name of the component of the edge.
 * - `annos_ns` - Namespace of the new annotation.
 * - `annos_name` - Name of the new annotation.
 * - `annos_value` - Value of the new annotation.
 * - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
 */
void annis_graphupdate_add_edge_label(AnnisGraphUpdate *ptr,
                                      const char *source_node,
                                      const char *target_node,
                                      const char *layer,
                                      const char *component_type,
                                      const char *component_name,
                                      const char *anno_ns,
                                      const char *anno_name,
                                      const char *anno_value,
                                      AnnisErrorList **err);

/**
 * Add "add node" action to the graph update object.
 *
 * - `ptr` - The graph update object.
 * - `node_name` - Name of the new node.
 * - `node_type` - Type of the new node, e.g. "node" or "corpus".
 * - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
 */
void annis_graphupdate_add_node(AnnisGraphUpdate *ptr,
                                const char *node_name,
                                const char *node_type,
                                AnnisErrorList **err);

/**
 * Add "add node label" action to the graph update object.
 *
 * - `ptr` - The graph update object.
 * - `node_name` - Name of the node the label is attached to.
 * - `annos_ns` - Namespace of the new annotation.
 * - `annos_name` - Name of the new annotation.
 * - `annos_value` - Value of the new annotation.
 * - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
 */
void annis_graphupdate_add_node_label(AnnisGraphUpdate *ptr,
                                      const char *node_name,
                                      const char *anno_ns,
                                      const char *anno_name,
                                      const char *anno_value,
                                      AnnisErrorList **err);

/**
 * Add "delete edge" action to the graph update object.
 *
 * - `ptr` - The graph update object.
 * - `source_node` - Name of source node of the edge to delete.
 * - `target_node` - Name of target node of the edge to delete.
 * - `layer` - Layer of the edge to delete.
 * - `component_type` - Type of the component of the edge to delete.
 * - `component_name` - Name of the component of the edge to delete.
 * - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
 */
void annis_graphupdate_delete_edge(AnnisGraphUpdate *ptr,
                                   const char *source_node,
                                   const char *target_node,
                                   const char *layer,
                                   const char *component_type,
                                   const char *component_name,
                                   AnnisErrorList **err);

/**
 * Add "delete edge label" action to the graph update object.
 *
 * - `ptr` - The graph update object.
 * - `source_node` - Name of source node of the edge.
 * - `target_node` - Name of target node of the edge.
 * - `layer` - Layer of the edge.
 * - `component_type` - Type of the component of the edge.
 * - `component_name` - Name of the component of the edge.
 * - `annos_ns` - Namespace of the annotation to delete.
 * - `annos_name` - Name of the annotation to delete.
 * - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
 */
void annis_graphupdate_delete_edge_label(AnnisGraphUpdate *ptr,
                                         const char *source_node,
                                         const char *target_node,
                                         const char *layer,
                                         const char *component_type,
                                         const char *component_name,
                                         const char *anno_ns,
                                         const char *anno_name,
                                         AnnisErrorList **err);

/**
 * Add "delete node" action to the graph update object.
 *
 * - `ptr` - The graph update object.
 * - `node_name` - Name of node to delete.
 * - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
 */
void annis_graphupdate_delete_node(AnnisGraphUpdate *ptr,
                                   const char *node_name,
                                   AnnisErrorList **err);

/**
 * Add "delete node label" action to the graph update object.
 *
 * - `ptr` - The graph update object.
 * - `node_name` - Name of the node the label is attached to.
 * - `annos_ns` - Namespace of deleted new annotation.
 * - `annos_name` - Name of the deleted annotation.
 * - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
 */
void annis_graphupdate_delete_node_label(AnnisGraphUpdate *ptr,
                                         const char *node_name,
                                         const char *anno_ns,
                                         const char *anno_name,
                                         AnnisErrorList **err);

/**
 * Create a new graph (empty) update instance
 */
AnnisGraphUpdate *annis_graphupdate_new(void);

/**
 * Initialize the logging of this library.
 *
 * - `logfile` - The file that is used to output the log messages.
 * - `level` - Minimum level to output.
 * - `err` - Pointer to a list of errors. If any error occured, this list will be non-empty.
 */
void annis_init_logging(const char *logfile, AnnisLogLevel level, AnnisErrorList **err);

/**
 * Returns a pointer to the next node ID for the iterator given by the `ptr` argument
 * or `NULL` if iterator is empty.
 */
AnnisNodeID *annis_iter_nodeid_next(AnnisIterPtr_NodeID *ptr);

/**
 * Get a read-only reference to the string at the at position (`row`, `col`) of the matrix.
 */
const char *annis_matrix_str_get(const AnnisMatrix_CString *ptr, size_t row, size_t col);

/**
 * Returns the number of columns of the string matrix.
 */
size_t annis_matrix_str_ncols(const AnnisMatrix_CString *ptr);

/**
 * Returns the number of rows of the string matrix.
 */
size_t annis_matrix_str_nrows(const AnnisMatrix_CString *ptr);

/**
 * Frees the string given as `s` argument.
 */
void annis_str_free(char *s);

/**
 * Get a read-only reference to the annotation at position `i` of the vector.
 */
const AnnisAnnotation *annis_vec_annotation_get(const AnnisVec_Annotation *ptr, size_t i);

/**
 * Returns the number of elements of the annotation vector.
 */
size_t annis_vec_annotation_size(const AnnisVec_Annotation *ptr);

/**
 * Get a read-only reference to the component at position `i` of the vector.
 */
const AnnisAnnotationComponent *annis_vec_component_get(const AnnisVec_AnnotationComponent *ptr,
                                                        size_t i);

/**
 * Returns the number of elements of the component vector.
 */
size_t annis_vec_component_size(const AnnisVec_AnnotationComponent *ptr);

/**
 * Get a read-only reference to the edge at position `i` of the vector.
 */
const AnnisEdge *annis_vec_edge_get(const AnnisVec_Edge *ptr, size_t i);

/**
 * Returns the number of elements of the edge vector.
 */
size_t annis_vec_edge_size(const AnnisVec_Edge *ptr);

/**
 * Create a string representing the annotation name part of the query attribute description.
 *
 * The resulting char* must be freeed with annis_str_free!
 */
char *annis_vec_qattdesc_get_anno_name(const AnnisVec_QueryAttributeDescription *ptr, size_t i);

/**
 * Create a string representing the AQL fragment part of the query attribute description.
 *
 * The resulting char* must be freeed with annis_str_free!
 */
char *annis_vec_qattdesc_get_aql_fragment(const AnnisVec_QueryAttributeDescription *ptr, size_t i);

/**
 * Get a read-only reference to the query attribute description at position `i` of the vector.
 */
uintptr_t annis_vec_qattdesc_get_component_nr(const AnnisVec_QueryAttributeDescription *ptr,
                                              size_t i);

/**
 * Create a string representing the variable part of the query attribute description.
 *
 * The resulting char* must be freeed with annis_str_free!
 */
char *annis_vec_qattdesc_get_variable(const AnnisVec_QueryAttributeDescription *ptr, size_t i);

/**
 * Returns the number of elements of the query attribute description vector.
 */
size_t annis_vec_qattdesc_size(const AnnisVec_QueryAttributeDescription *ptr);

/**
 * Get a read-only reference to the string at position `i` of the vector.
 */
const char *annis_vec_str_get(const AnnisVec_CString *ptr, size_t i);

/**
 * Create a new string vector.
 */
AnnisVec_CString *annis_vec_str_new(void);

/**
 * Add an element to the string vector.
 */
void annis_vec_str_push(AnnisVec_CString *ptr, const char *v);

/**
 * Returns the number of elements of the string vector.
 */
size_t annis_vec_str_size(const AnnisVec_CString *ptr);

#endif /* graphannis_capi_h */
