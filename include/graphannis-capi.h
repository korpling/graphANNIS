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

/* Generated with cbindgen:0.6.7 */

#include <stdint.h>
#include <stdlib.h>
#include <stdbool.h>

/*
 * Specifies the type of component. Types determine certain semantics about the edges of this graph components.
 */
typedef enum {
  /*
   * Edges between a span node and its tokens. Implies text coverage.
   */
  Coverage,
  /*
   * Edges between a structural node and any other structural node, span or token. Implies text coverage.
   */
  Dominance = 2,
  /*
   * Edge between any node.
   */
  Pointing,
  /*
   * Edge between two tokens implying that the source node comes before the target node in the textflow.
   */
  Ordering,
  /*
   * Explicit edge between any non-token node and the left-most token it covers.
   */
  LeftToken,
  /*
   * Explicit edge between any non-token node and the right-most token it covers.
   */
  RightToken,
  /*
   * Implies that the source node belongs to the parent corpus/subcorpus/document/datasource node.
   */
  PartOf,
} AnnisComponentType;

/*
 * An enum of all supported input formats of graphANNIS.
 */
typedef enum {
  /*
   * Legacy [relANNIS import file format](http://korpling.github.io/ANNIS/doc/dev-annisimportformat.html)
   */
  RelANNIS,
} AnnisImportFormat;

typedef enum {
  Off,
  Error,
  Warn,
  Info,
  Debug,
  Trace,
} AnnisLogLevel;

/*
 * An enum over all supported query languages of graphANNIS.
 *
 * Currently, only the ANNIS Query Language (AQL) and its variants are supported, but this enum allows us to add a support for older query language versions
 * or completly new query languages.
 */
typedef enum {
  AQL,
  /*
   * Emulates the (sometimes problematic) behavior of AQL used in ANNIS 3
   */
  AQLQuirksV3,
} AnnisQueryLanguage;

/*
 * Defines the order of results of a `find` query.
 */
typedef enum {
  /*
   * Order results by their document name and the the text position of the match.
   */
  Normal,
  /*
   * Inverted the order of `Normal`.
   */
  Inverted,
  /*
   * A random ordering which is **not stable**. Each new query will result in a different order.
   */
  Randomized,
  /*
   * Results are not ordered at all, but also not actively randomized
   * Each new query *might* result in a different order.
   */
  NotSorted,
} AnnisResultOrder;

/*
 * An annotation with a qualified name and a value.
 */
typedef struct AnnisAnnotation AnnisAnnotation;

/*
 * Identifies an edge component of the graph.
 */
typedef struct AnnisComponent AnnisComponent;

/*
 * A thread-safe API for managing corpora stored in a common location on the file system.
 *
 * Multiple corpora can be part of a corpus storage and they are identified by their unique name.
 * Corpora are loaded from disk into main memory on demand:
 * An internal main memory cache is used to avoid re-loading a recently queried corpus from disk again.
 */
typedef struct AnnisCorpusStorage AnnisCorpusStorage;

/*
 * Definition of the result of a `frequency` query.
 *
 * This is a vector of rows, and each row is a vector of columns with the different
 * attribute values and a number of matches having this combination of attribute values.
 */
typedef struct AnnisFrequencyTable_CString AnnisFrequencyTable_CString;

/*
 * A representation of a graph including node annotations and edges.
 * Edges are partioned into components and each component is implemented by specialized graph storage implementation.
 *
 * Use the [CorpusStorage](struct.CorpusStorage.html) struct to create and manage instances of a `Graph`.
 *
 * Graphs can have an optional location on the disk.
 * In this case, changes to the graph via the [apply_update(...)](#method.apply_update) function are automatically persisted to this location.
 *
 */
typedef struct AnnisGraph AnnisGraph;

/*
 * A list of changes to apply to an graph.
 */
typedef struct AnnisGraphUpdate AnnisGraphUpdate;

typedef struct AnnisIterPtr_NodeID AnnisIterPtr_NodeID;

typedef struct AnnisVec_Annotation AnnisVec_Annotation;

typedef struct AnnisVec_CString AnnisVec_CString;

typedef struct AnnisVec_Component AnnisVec_Component;

typedef struct AnnisVec_Edge AnnisVec_Edge;

typedef struct AnnisVec_Error AnnisVec_Error;

typedef struct AnnisVec_QueryAttributeDescription AnnisVec_QueryAttributeDescription;

typedef struct AnnisVec_Vec_CString AnnisVec_Vec_CString;

typedef AnnisVec_Error AnnisErrorList;

/*
 * A struct that contains the extended results of the count query.
 */
typedef struct {
  /*
   * Total number of matches.
   */
  uint64_t match_count;
  /*
   * Number of documents with at least one match.
   */
  uint64_t document_count;
} AnnisCountExtra;

/*
 * Simple definition of a matrix from a single data type.
 */
typedef AnnisVec_Vec_CString AnnisMatrix_CString;

/*
 * Unique internal identifier for a single node.
 */
typedef uint64_t AnnisNodeID;

/*
 * Directed edge between a source and target node which are identified by their ID.
 */
typedef struct {
  AnnisNodeID source;
  AnnisNodeID target;
} AnnisEdge;

char *annis_annotation_name(const AnnisAnnotation *ptr);

char *annis_annotation_ns(const AnnisAnnotation *ptr);

char *annis_annotation_val(const AnnisAnnotation *ptr);

char *annis_component_layer(const AnnisComponent *c);

char *annis_component_name(const AnnisComponent *c);

AnnisComponentType annis_component_type(const AnnisComponent *c);

void annis_cs_apply_update(AnnisCorpusStorage *ptr,
                           const char *corpus,
                           AnnisGraphUpdate *update,
                           AnnisErrorList **err);

AnnisGraph *annis_cs_corpus_graph(const AnnisCorpusStorage *ptr,
                                  const char *corpus_name,
                                  AnnisErrorList **err);

uint64_t annis_cs_count(const AnnisCorpusStorage *ptr,
                        const char *corpus,
                        const char *query,
                        AnnisQueryLanguage query_language,
                        AnnisErrorList **err);

AnnisCountExtra annis_cs_count_extra(const AnnisCorpusStorage *ptr,
                                     const char *corpus,
                                     const char *query,
                                     AnnisQueryLanguage query_language,
                                     AnnisErrorList **err);

/*
 * Deletes a corpus from the corpus storage.
 */
bool annis_cs_delete(AnnisCorpusStorage *ptr, const char *corpus, AnnisErrorList **err);

AnnisVec_CString *annis_cs_find(const AnnisCorpusStorage *ptr,
                                const char *corpus_name,
                                const char *query,
                                AnnisQueryLanguage query_language,
                                size_t offset,
                                size_t limit,
                                AnnisResultOrder order,
                                AnnisErrorList **err);

void annis_cs_free(AnnisCorpusStorage *ptr);

AnnisFrequencyTable_CString *annis_cs_frequency(const AnnisCorpusStorage *ptr,
                                                const char *corpus_name,
                                                const char *query,
                                                AnnisQueryLanguage query_language,
                                                const char *frequency_query_definition,
                                                AnnisErrorList **err);

char *annis_cs_import_from_fs(AnnisCorpusStorage *ptr,
                              const char *path,
                              AnnisImportFormat format,
                              const char *corpus,
                              AnnisErrorList **err);

/*
 * List all known corpora.
 */
AnnisVec_CString *annis_cs_list(const AnnisCorpusStorage *ptr, AnnisErrorList **err);

AnnisVec_Component *annis_cs_list_components_by_type(AnnisCorpusStorage *ptr,
                                                     const char *corpus_name,
                                                     AnnisComponentType ctype);

AnnisMatrix_CString *annis_cs_list_edge_annotations(const AnnisCorpusStorage *ptr,
                                                    const char *corpus_name,
                                                    AnnisComponentType component_type,
                                                    const char *component_name,
                                                    const char *component_layer,
                                                    bool list_values,
                                                    bool only_most_frequent_values);

AnnisMatrix_CString *annis_cs_list_node_annotations(const AnnisCorpusStorage *ptr,
                                                    const char *corpus_name,
                                                    bool list_values,
                                                    bool only_most_frequent_values);

AnnisVec_QueryAttributeDescription *annis_cs_node_descriptions(const AnnisCorpusStorage *ptr,
                                                               const char *query,
                                                               AnnisQueryLanguage query_language,
                                                               AnnisErrorList **err);

AnnisGraph *annis_cs_subcorpus_graph(const AnnisCorpusStorage *ptr,
                                     const char *corpus_name,
                                     const AnnisVec_CString *corpus_ids,
                                     AnnisErrorList **err);

AnnisGraph *annis_cs_subgraph(const AnnisCorpusStorage *ptr,
                              const char *corpus_name,
                              const AnnisVec_CString *node_ids,
                              size_t ctx_left,
                              size_t ctx_right,
                              AnnisErrorList **err);

AnnisGraph *annis_cs_subgraph_for_query(const AnnisCorpusStorage *ptr,
                                        const char *corpus_name,
                                        const char *query,
                                        AnnisQueryLanguage query_language,
                                        AnnisErrorList **err);

AnnisGraph *annis_cs_subgraph_for_query_with_ctype(const AnnisCorpusStorage *ptr,
                                                   const char *corpus_name,
                                                   const char *query,
                                                   AnnisQueryLanguage query_language,
                                                   AnnisComponentType component_type_filter,
                                                   AnnisErrorList **err);

bool annis_cs_validate_query(const AnnisCorpusStorage *ptr,
                             const char *corpus,
                             const char *query,
                             AnnisQueryLanguage query_language,
                             AnnisErrorList **err);

/*
 * Create a new corpus storage with an automatically determined maximum cache size.
 */
AnnisCorpusStorage *annis_cs_with_auto_cache_size(const char *db_dir, bool use_parallel);

/*
 * Create a new corpus storage with an manually defined maximum cache size.
 */
AnnisCorpusStorage *annis_cs_with_max_cache_size(const char *db_dir,
                                                 uintptr_t max_cache_size,
                                                 bool use_parallel);

const char *annis_error_get_kind(const AnnisErrorList *ptr, size_t i);

const char *annis_error_get_msg(const AnnisErrorList *ptr, size_t i);

size_t annis_error_size(const AnnisErrorList *ptr);

void annis_free(void *ptr);

size_t annis_freqtable_str_count(const AnnisFrequencyTable_CString *ptr, size_t row);

const char *annis_freqtable_str_get(const AnnisFrequencyTable_CString *ptr, size_t row, size_t col);

size_t annis_freqtable_str_ncols(const AnnisFrequencyTable_CString *ptr);

size_t annis_freqtable_str_nrows(const AnnisFrequencyTable_CString *ptr);

AnnisVec_Component *annis_graph_all_components(const AnnisGraph *g);

AnnisVec_Component *annis_graph_all_components_by_type(const AnnisGraph *g,
                                                       AnnisComponentType ctype);

AnnisVec_Annotation *annis_graph_annotations_for_edge(const AnnisGraph *g,
                                                      AnnisEdge edge,
                                                      const AnnisComponent *component);

AnnisVec_Annotation *annis_graph_annotations_for_node(const AnnisGraph *g, AnnisNodeID node);

AnnisIterPtr_NodeID *annis_graph_nodes_by_type(const AnnisGraph *g, const char *node_type);

AnnisVec_Edge *annis_graph_outgoing_edges(const AnnisGraph *g,
                                          AnnisNodeID source,
                                          const AnnisComponent *component);

void annis_graphupdate_add_edge(AnnisGraphUpdate *ptr,
                                const char *source_node,
                                const char *target_node,
                                const char *layer,
                                const char *component_type,
                                const char *component_name);

void annis_graphupdate_add_edge_label(AnnisGraphUpdate *ptr,
                                      const char *source_node,
                                      const char *target_node,
                                      const char *layer,
                                      const char *component_type,
                                      const char *component_name,
                                      const char *anno_ns,
                                      const char *anno_name,
                                      const char *anno_value);

void annis_graphupdate_add_node(AnnisGraphUpdate *ptr,
                                const char *node_name,
                                const char *node_type);

void annis_graphupdate_add_node_label(AnnisGraphUpdate *ptr,
                                      const char *node_name,
                                      const char *anno_ns,
                                      const char *anno_name,
                                      const char *anno_value);

void annis_graphupdate_delete_edge(AnnisGraphUpdate *ptr,
                                   const char *source_node,
                                   const char *target_node,
                                   const char *layer,
                                   const char *component_type,
                                   const char *component_name);

void annis_graphupdate_delete_edge_label(AnnisGraphUpdate *ptr,
                                         const char *source_node,
                                         const char *target_node,
                                         const char *layer,
                                         const char *component_type,
                                         const char *component_name,
                                         const char *anno_ns,
                                         const char *anno_name);

void annis_graphupdate_delete_node(AnnisGraphUpdate *ptr, const char *node_name);

void annis_graphupdate_delete_node_label(AnnisGraphUpdate *ptr,
                                         const char *node_name,
                                         const char *anno_ns,
                                         const char *anno_name);

/*
 * Create a new graph update instance
 */
AnnisGraphUpdate *annis_graphupdate_new(void);

size_t annis_graphupdate_size(const AnnisGraphUpdate *ptr);

void annis_init_logging(const char *logfile, AnnisLogLevel level, AnnisErrorList **err);

AnnisNodeID *annis_iter_nodeid_next(AnnisIterPtr_NodeID *ptr);

const char *annis_matrix_str_get(const AnnisMatrix_CString *ptr, size_t row, size_t col);

size_t annis_matrix_str_ncols(const AnnisMatrix_CString *ptr);

size_t annis_matrix_str_nrows(const AnnisMatrix_CString *ptr);

void annis_str_free(char *s);

const AnnisAnnotation *annis_vec_annotation_get(const AnnisVec_Annotation *ptr, size_t i);

size_t annis_vec_annotation_size(const AnnisVec_Annotation *ptr);

const AnnisComponent *annis_vec_component_get(const AnnisVec_Component *ptr, size_t i);

size_t annis_vec_component_size(const AnnisVec_Component *ptr);

const AnnisEdge *annis_vec_edge_get(const AnnisVec_Edge *ptr, size_t i);

size_t annis_vec_edge_size(const AnnisVec_Edge *ptr);

/*
 * Result char* must be freeed with annis_str_free!
 */
char *annis_vec_qattdesc_get_anno_name(const AnnisVec_QueryAttributeDescription *ptr, size_t i);

/*
 * Result char* must be freeed with annis_str_free!
 */
char *annis_vec_qattdesc_get_aql_fragment(const AnnisVec_QueryAttributeDescription *ptr, size_t i);

uintptr_t annis_vec_qattdesc_get_component_nr(const AnnisVec_QueryAttributeDescription *ptr,
                                              size_t i);

/*
 * Result char* must be freeed with annis_str_free!
 */
char *annis_vec_qattdesc_get_variable(const AnnisVec_QueryAttributeDescription *ptr, size_t i);

size_t annis_vec_qattdesc_size(const AnnisVec_QueryAttributeDescription *ptr);

const char *annis_vec_str_get(const AnnisVec_CString *ptr, size_t i);

AnnisVec_CString *annis_vec_str_new(void);

void annis_vec_str_push(AnnisVec_CString *ptr, const char *v);

size_t annis_vec_str_size(const AnnisVec_CString *ptr);

#endif /* graphannis_capi_h */
