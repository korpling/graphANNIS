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

/* Generated with cbindgen:0.5.2 */

#include <stdint.h>
#include <stdlib.h>
#include <stdbool.h>

typedef struct AnnisCorpusStorage AnnisCorpusStorage;

typedef struct AnnisError AnnisError;

typedef struct AnnisGraphDB AnnisGraphDB;

typedef struct AnnisGraphUpdate AnnisGraphUpdate;

typedef struct AnnisIterPtr_AnnisNodeID AnnisIterPtr_AnnisNodeID;

typedef struct AnnisVec_AnnisAnnotation AnnisVec_AnnisAnnotation;

typedef struct AnnisVec_AnnisCString AnnisVec_AnnisCString;

typedef uint32_t AnnisNodeID;

typedef uint32_t AnnisStringID;

typedef struct {
  AnnisStringID name;
  AnnisStringID ns;
} AnnisAnnoKey;

typedef struct {
  AnnisAnnoKey key;
  AnnisStringID val;
} AnnisAnnotation;

AnnisError *annis_cs_apply_update(AnnisCorpusStorage *ptr,
                                  const char *corpus,
                                  AnnisGraphUpdate *update);

uint64_t annis_cs_count(const AnnisCorpusStorage *ptr,
                        const char *corpus,
                        const char *query_as_json);

AnnisVec_AnnisCString *annis_cs_find(const AnnisCorpusStorage *ptr,
                                     const char *corpus_name,
                                     const char *query_as_json,
                                     size_t offset,
                                     size_t limit);

/*
 * List all known corpora.
 */
AnnisVec_AnnisCString *annis_cs_list(const AnnisCorpusStorage *ptr);

/*
 * Create a new corpus storage
 */
AnnisCorpusStorage *annis_cs_new(const char *db_dir);

AnnisGraphDB *annis_cs_subgraph(const AnnisCorpusStorage *ptr,
                                const char *corpus_name,
                                const AnnisVec_AnnisCString *node_ids,
                                size_t ctx_left,
                                size_t ctx_right);

const char *annis_error_get_msg(const AnnisError *ptr);

void annis_free(void *ptr);

AnnisVec_AnnisAnnotation *annis_graph_node_labels(const AnnisGraphDB *g, AnnisNodeID node);

AnnisIterPtr_AnnisNodeID *annis_graph_nodes_by_type(const AnnisGraphDB *g, const char *node_type);

char *annis_graph_str(const AnnisGraphDB *g, AnnisStringID str_id);

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

const AnnisNodeID *annis_iter_nodeid_next(AnnisIterPtr_AnnisNodeID *ptr);

void annis_str_free(char *s);

const AnnisAnnotation *annis_vec_annotation_get(const AnnisVec_AnnisAnnotation *ptr, size_t i);

size_t annis_vec_annotation_size(const AnnisVec_AnnisAnnotation *ptr);

const char *annis_vec_str_get(const AnnisVec_AnnisCString *ptr, size_t i);

size_t annis_vec_str_size(const AnnisVec_AnnisCString *ptr);

#endif /* graphannis_capi_h */
