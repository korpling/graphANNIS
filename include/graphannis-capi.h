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

/* Generated with cbindgen:0.6.1 */

#include <stdint.h>
#include <stdlib.h>
#include <stdbool.h>

typedef enum {
  Coverage,
  InverseCoverage,
  Dominance,
  Pointing,
  Ordering,
  LeftToken,
  RightToken,
  PartOfSubcorpus,
} AnnisComponentType;

typedef enum {
  Off,
  Error,
  Warn,
  Info,
  Debug,
  Trace,
} AnnisLogLevel;

typedef enum {
  AQL,
} AnnisQueryLanguage;

typedef enum {
  Normal,
  Inverted,
  Random,
} AnnisResultOrder;

typedef struct AnnisComponent AnnisComponent;

/*
 * A thread-safe API for managing corpora stored in a common location on the file system.
 *
 * Multiple corpora can be part of a corpus storage and they are identified by their unique name.
 * Corpora are loaded from disk into main memory on demand:
 * An internal main memory cache is used to avoid re-loading a recently queried corpus from disk again.
 */
typedef struct AnnisCorpusStorage AnnisCorpusStorage;

typedef struct AnnisFrequencyTable_AnnisCString AnnisFrequencyTable_AnnisCString;

/*
 * A representation of a graph including node annotations and edges.
 * Edges are partioned into [components](types/struct.Component.html)
 * and each component is implemented by specialized [graph storage](trait.GraphStorage.html) implementation.
 *
 * Use the [CorpusStorage](struct.CorpusStorage.html) struct to create and manage instances of a `Graph`.
 *
 * Graphs can have an optional location on the disk.
 * In this case, changes to the graph via the [apply_update(...)](#method.apply_update) function are automatically persisted to this location.
 *
 */
typedef struct AnnisGraph AnnisGraph;

typedef struct AnnisGraphUpdate AnnisGraphUpdate;

typedef struct AnnisIterPtr_AnnisNodeID AnnisIterPtr_AnnisNodeID;

typedef struct AnnisString AnnisString;

typedef struct AnnisVec_AnnisAnnotation AnnisVec_AnnisAnnotation;

typedef struct AnnisVec_AnnisCString AnnisVec_AnnisCString;

typedef struct AnnisVec_AnnisComponent AnnisVec_AnnisComponent;

typedef struct AnnisVec_AnnisEdge AnnisVec_AnnisEdge;

typedef struct AnnisVec_AnnisError AnnisVec_AnnisError;

typedef struct AnnisVec_AnnisNodeDesc AnnisVec_AnnisNodeDesc;

typedef struct AnnisVec_AnnisVec_AnnisT AnnisVec_AnnisVec_AnnisT;

typedef AnnisVec_AnnisError AnnisErrorList;

typedef struct {
  uint64_t match_count;
  uint64_t document_count;
} AnnisCountExtra;

/*
 * Very simple definition of a matrix from a single data type. Not optimized at all.
 * TODO: Maybe a sparse matrix could be used.
 */
typedef AnnisVec_AnnisVec_AnnisT AnnisMatrix_AnnisCString;

typedef uint64_t AnnisNodeID;

typedef struct {
  AnnisNodeID source;
  AnnisNodeID target;
} AnnisEdge;

typedef struct {
  AnnisString name;
  AnnisString ns;
} AnnisAnnoKey;

typedef struct {
  AnnisAnnoKey key;
  AnnisString val;
} AnnisAnnotation;

char *annis_component_layer(const AnnisComponent *c);

char *annis_component_name(const AnnisComponent *c);

AnnisComponentType annis_component_type(const AnnisComponent *c);

AnnisVec_AnnisComponent *annis_cs_all_components_by_type(AnnisCorpusStorage *ptr,
                                                         const char *corpus_name,
                                                         AnnisComponentType ctype);

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

bool annis_cs_delete(AnnisCorpusStorage *ptr, const char *corpus, AnnisErrorList **err);

AnnisVec_AnnisCString *annis_cs_find(const AnnisCorpusStorage *ptr,
                                     const char *corpus_name,
                                     const char *query,
                                     AnnisQueryLanguage query_language,
                                     size_t offset,
                                     size_t limit,
                                     AnnisResultOrder order,
                                     AnnisErrorList **err);

void annis_cs_free(AnnisCorpusStorage *ptr);

AnnisFrequencyTable_AnnisCString *annis_cs_frequency(const AnnisCorpusStorage *ptr,
                                                     const char *corpus_name,
                                                     const char *query,
                                                     AnnisQueryLanguage query_language,
                                                     const char *frequency_query_definition,
                                                     AnnisErrorList **err);

void annis_cs_import_relannis(AnnisCorpusStorage *ptr,
                              const char *corpus,
                              const char *path,
                              AnnisErrorList **err);

/*
 * List all known corpora.
 */
AnnisVec_AnnisCString *annis_cs_list(const AnnisCorpusStorage *ptr, AnnisErrorList **err);

AnnisMatrix_AnnisCString *annis_cs_list_edge_annotations(const AnnisCorpusStorage *ptr,
                                                         const char *corpus_name,
                                                         AnnisComponentType component_type,
                                                         const char *component_name,
                                                         const char *component_layer,
                                                         bool list_values,
                                                         bool only_most_frequent_values);

AnnisMatrix_AnnisCString *annis_cs_list_node_annotations(const AnnisCorpusStorage *ptr,
                                                         const char *corpus_name,
                                                         bool list_values,
                                                         bool only_most_frequent_values);

/*
 * Create a new corpus storage
 */
AnnisCorpusStorage *annis_cs_new(const char *db_dir, bool use_parallel);

AnnisVec_AnnisNodeDesc *annis_cs_node_descriptions(const AnnisCorpusStorage *ptr,
                                                   const char *query,
                                                   AnnisQueryLanguage query_language,
                                                   AnnisErrorList **err);

AnnisGraph *annis_cs_subcorpus_graph(const AnnisCorpusStorage *ptr,
                                     const char *corpus_name,
                                     const AnnisVec_AnnisCString *corpus_ids,
                                     AnnisErrorList **err);

AnnisGraph *annis_cs_subgraph(const AnnisCorpusStorage *ptr,
                              const char *corpus_name,
                              const AnnisVec_AnnisCString *node_ids,
                              size_t ctx_left,
                              size_t ctx_right,
                              AnnisErrorList **err);

AnnisGraph *annis_cs_subgraph_for_query(const AnnisCorpusStorage *ptr,
                                        const char *corpus_name,
                                        const char *query,
                                        AnnisQueryLanguage query_language,
                                        AnnisErrorList **err);

bool annis_cs_validate_query(const AnnisCorpusStorage *ptr,
                             const char *corpus,
                             const char *query,
                             AnnisQueryLanguage query_language,
                             AnnisErrorList **err);

const char *annis_error_get_kind(const AnnisErrorList *ptr, size_t i);

const char *annis_error_get_msg(const AnnisErrorList *ptr, size_t i);

size_t annis_error_size(const AnnisErrorList *ptr);

void annis_free(void *ptr);

size_t annis_freqtable_str_count(const AnnisFrequencyTable_AnnisCString *ptr, size_t row);

const char *annis_freqtable_str_get(const AnnisFrequencyTable_AnnisCString *ptr,
                                    size_t row,
                                    size_t col);

size_t annis_freqtable_str_ncols(const AnnisFrequencyTable_AnnisCString *ptr);

size_t annis_freqtable_str_nrows(const AnnisFrequencyTable_AnnisCString *ptr);

AnnisVec_AnnisComponent *annis_graph_all_components(const AnnisGraph *g);

AnnisVec_AnnisComponent *annis_graph_all_components_by_type(const AnnisGraph *g,
                                                            AnnisComponentType ctype);

void annis_graph_apply_update(AnnisGraph *g, AnnisGraphUpdate *update, AnnisErrorList **err);

AnnisVec_AnnisAnnotation *annis_graph_edge_labels(const AnnisGraph *g,
                                                  AnnisEdge edge,
                                                  const AnnisComponent *component);

AnnisVec_AnnisAnnotation *annis_graph_node_labels(const AnnisGraph *g, AnnisNodeID node);

AnnisIterPtr_AnnisNodeID *annis_graph_nodes_by_type(const AnnisGraph *g, const char *node_type);

AnnisVec_AnnisEdge *annis_graph_outgoing_edges(const AnnisGraph *g,
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

AnnisNodeID *annis_iter_nodeid_next(AnnisIterPtr_AnnisNodeID *ptr);

const char *annis_matrix_str_get(const AnnisMatrix_AnnisCString *ptr, size_t row, size_t col);

size_t annis_matrix_str_ncols(const AnnisMatrix_AnnisCString *ptr);

size_t annis_matrix_str_nrows(const AnnisMatrix_AnnisCString *ptr);

void annis_str_free(char *s);

const AnnisAnnotation *annis_vec_annotation_get(const AnnisVec_AnnisAnnotation *ptr, size_t i);

size_t annis_vec_annotation_size(const AnnisVec_AnnisAnnotation *ptr);

const AnnisComponent *annis_vec_component_get(const AnnisVec_AnnisComponent *ptr, size_t i);

size_t annis_vec_component_size(const AnnisVec_AnnisComponent *ptr);

const AnnisEdge *annis_vec_edge_get(const AnnisVec_AnnisEdge *ptr, size_t i);

size_t annis_vec_edge_size(const AnnisVec_AnnisEdge *ptr);

/*
 * Result char* must be freeed with annis_str_free!
 */
char *annis_vec_nodedesc_get_anno_name(const AnnisVec_AnnisNodeDesc *ptr, size_t i);

/*
 * Result char* must be freeed with annis_str_free!
 */
char *annis_vec_nodedesc_get_aql_fragment(const AnnisVec_AnnisNodeDesc *ptr, size_t i);

uintptr_t annis_vec_nodedesc_get_component_nr(const AnnisVec_AnnisNodeDesc *ptr, size_t i);

/*
 * Result char* must be freeed with annis_str_free!
 */
char *annis_vec_nodedesc_get_variable(const AnnisVec_AnnisNodeDesc *ptr, size_t i);

size_t annis_vec_nodedesc_size(const AnnisVec_AnnisNodeDesc *ptr);

const char *annis_vec_str_get(const AnnisVec_AnnisCString *ptr, size_t i);

AnnisVec_AnnisCString *annis_vec_str_new(void);

void annis_vec_str_push(AnnisVec_AnnisCString *ptr, const char *v);

size_t annis_vec_str_size(const AnnisVec_AnnisCString *ptr);

#endif /* graphannis_capi_h */
