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

/* Generated with cbindgen:0.5.0 */

#include <stdint.h>
#include <stdlib.h>
#include <stdbool.h>

typedef struct AnnisCorpusStorage AnnisCorpusStorage;

typedef struct AnnisGraphUpdate AnnisGraphUpdate;

typedef struct {
  char *error_msg;
} AnnisOptError;

AnnisOptError annis_cs_apply_update(AnnisCorpusStorage *ptr,
                                    const char *corpus,
                                    AnnisGraphUpdate *update);

uint64_t annis_cs_count(const AnnisCorpusStorage *ptr,
                        const char *corpus,
                        const char *query_as_json);

/*
 * Delete a corpus storage
 */
void annis_cs_free(AnnisCorpusStorage *ptr);

/*
 * Return an NULL-terminated array of strings that contains the names of all known corpora.
 */
char **annis_cs_list(const AnnisCorpusStorage *ptr);

/*
 * Create a new corpus storage
 */
AnnisCorpusStorage *annis_cs_new(const char *db_dir);

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
 * Delete a graph update instance
 */
void annis_graphupdate_free(AnnisGraphUpdate *ptr);

/*
 * Create a new graph update instance
 */
AnnisGraphUpdate *annis_graphupdate_new(void);

void annis_str_free(char *s);

#endif /* graphannis_capi_h */
