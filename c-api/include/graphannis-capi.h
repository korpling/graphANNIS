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

typedef struct ANNIS_CorpusStorage ANNIS_CorpusStorage;

typedef struct ANNIS_GraphUpdate ANNIS_GraphUpdate;

typedef struct {
  bool is_error;
  const char *error_msg;
} ANNIS_OptError;

typedef enum {
  AddNode,
  DeleteNode,
  AddNodeLabel,
  DeleteNodeLabel,
  AddEdge,
  DeleteEdge,
  AddEdgeLabel,
  DeleteEdgeLabel,
} ANNIS_UpdateEvent_Tag;

typedef struct {
  const char *node_name;
  const char *node_type;
} ANNIS_AddNode_Body;

typedef struct {
  const char *node_name;
} ANNIS_DeleteNode_Body;

typedef struct {
  const char *node_name;
  const char *anno_ns;
  const char *anno_name;
  const char *anno_value;
} ANNIS_AddNodeLabel_Body;

typedef struct {
  const char *node_name;
  const char *anno_ns;
  const char *anno_name;
} ANNIS_DeleteNodeLabel_Body;

typedef struct {
  const char *source_node;
  const char *target_node;
  const char *layer;
  const char *component_type;
  const char *component_name;
} ANNIS_AddEdge_Body;

typedef struct {
  const char *source_node;
  const char *target_node;
  const char *layer;
  const char *component_type;
  const char *component_name;
} ANNIS_DeleteEdge_Body;

typedef struct {
  const char *source_node;
  const char *target_node;
  const char *layer;
  const char *component_type;
  const char *component_name;
  const char *anno_ns;
  const char *anno_name;
  const char *anno_value;
} ANNIS_AddEdgeLabel_Body;

typedef struct {
  const char *source_node;
  const char *target_node;
  const char *layer;
  const char *component_type;
  const char *component_name;
  const char *anno_ns;
  const char *anno_name;
} ANNIS_DeleteEdgeLabel_Body;

typedef struct {
  ANNIS_UpdateEvent_Tag tag;
  union {
    ANNIS_AddNode_Body add_node;
    ANNIS_DeleteNode_Body delete_node;
    ANNIS_AddNodeLabel_Body add_node_label;
    ANNIS_DeleteNodeLabel_Body delete_node_label;
    ANNIS_AddEdge_Body add_edge;
    ANNIS_DeleteEdge_Body delete_edge;
    ANNIS_AddEdgeLabel_Body add_edge_label;
    ANNIS_DeleteEdgeLabel_Body delete_edge_label;
  };
} ANNIS_UpdateEvent;

ANNIS_OptError annis_cs_apply_update(ANNIS_CorpusStorage *ptr,
                                     const char *corpus,
                                     ANNIS_GraphUpdate *update);

uint64_t annis_cs_count(const ANNIS_CorpusStorage *ptr,
                        const char *corpus,
                        const char *query_as_json);

/*
 * Delete a corpus storage
 */
void annis_cs_free(ANNIS_CorpusStorage *ptr);

/*
 * Create a new corpus storage
 */
ANNIS_CorpusStorage *annis_cs_new(const char *db_dir);

void annis_graphupdate_add_event(ANNIS_GraphUpdate *ptr, ANNIS_UpdateEvent event);

void annis_graphupdate_add_node(ANNIS_GraphUpdate *ptr,
                                const char *node_name,
                                const char *node_type);

/*
 * Delete a graph update instance
 */
void annis_graphupdate_free(ANNIS_GraphUpdate *ptr);

/*
 * Create a new graph update instance
 */
ANNIS_GraphUpdate *annis_graphupdate_new(void);

#endif /* graphannis_capi_h */
