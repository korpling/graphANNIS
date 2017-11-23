
#ifndef cheddar_generated_annis_graphanniscapi_h
#define cheddar_generated_annis_graphanniscapi_h


#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stdbool.h>



/**
A non-null terminated string.
 */
typedef struct annis_String {
	char const* s;
	size_t length;
} annis_String;

typedef struct annis_Option_String {
	bool valid;
	annis_String value;
} annis_Option_String;

typedef struct annis_Option_u32 {
	bool valid;
	uint32_t value;
} annis_Option_u32;

typedef annis_Option_u32 annis_Option_StringID;

typedef struct annis_Vec_Annotation {
	Annotation const* v;
	size_t length;
} annis_Vec_Annotation;

typedef struct annis_MatchIt annis_MatchIt;

typedef struct annis_Option_Match {
	bool valid;
	NodeID node;
	Annotation anno;
} annis_Option_Match;

void annis_matchit_free(annis_MatchIt* ptr);

annis_Option_Match annis_matchit_next(annis_MatchIt* ptr);



typedef struct annis_StringStoragePtr annis_StringStoragePtr;

annis_StringStoragePtr* annis_stringstorage_new(void);

void annis_stringstorage_free(annis_StringStoragePtr* ptr);

annis_Option_String annis_stringstorage_str(annis_StringStoragePtr const* ptr, uint32_t id);

annis_Option_u32 annis_stringstorage_find_id(annis_StringStoragePtr const* ptr, char const* value);

uint32_t annis_stringstorage_add(annis_StringStoragePtr* ptr, char const* value);

void annis_stringstorage_clear(annis_StringStoragePtr* ptr);

size_t annis_stringstorage_len(annis_StringStoragePtr const* ptr);

double annis_stringstorage_avg_length(annis_StringStoragePtr const* ptr);

void annis_stringstorage_save_to_file(annis_StringStoragePtr const* ptr, char const* path);

void annis_stringstorage_load_from_file(annis_StringStoragePtr* ptr, char const* path);

size_t annis_stringstorage_estimate_memory(annis_StringStoragePtr const* ptr);



typedef struct annis_ASNodePtr annis_ASNodePtr;

typedef struct annis_ASEdgePtr annis_ASEdgePtr;

annis_ASNodePtr* annis_asnode_new(void);

void annis_asnode_free(annis_ASNodePtr* ptr);

void annis_asnode_insert(annis_ASNodePtr* ptr, NodeID item, Annotation anno);

annis_Option_StringID annis_asnode_remove(annis_ASNodePtr* ptr, NodeID item, AnnoKey key);

size_t annis_asnode_len(annis_ASNodePtr const* ptr);

annis_Option_StringID annis_asnode_get(annis_ASNodePtr const* ptr, NodeID item, AnnoKey key);

annis_Vec_Annotation annis_asnode_get_all(annis_ASNodePtr const* ptr, NodeID item);

size_t annis_asnode_guess_max_count(annis_ASNodePtr const* ptr, annis_Option_StringID ns, StringID name, char const* lower_val, char const* upper_val);

size_t annis_asnode_guess_max_count_regex(annis_ASNodePtr const* ptr, annis_Option_StringID ns, StringID name, char const* pattern);

void annis_asnode_calculate_statistics(annis_ASNodePtr* ptr, annis_StringStoragePtr const* stringstorage);

annis_MatchIt* annis_asnode_exact_anno_search(annis_ASNodePtr const* ptr, annis_Option_StringID ns, StringID name, annis_Option_StringID value);

annis_MatchIt* annis_asnode_regex_anno_search(annis_ASNodePtr const* ptr, annis_StringStoragePtr const* strings_ptr, annis_Option_StringID ns, StringID name, char const* pattern);

annis_ASEdgePtr* annis_asedge_new(void);

void annis_asedge_free(annis_ASEdgePtr* ptr);

void annis_asedge_insert(annis_ASEdgePtr* ptr, Edge item, Annotation anno);

annis_Option_StringID annis_asedge_remove(annis_ASEdgePtr* ptr, Edge item, AnnoKey key);

size_t annis_asedge_len(annis_ASEdgePtr const* ptr);

annis_Option_StringID annis_asedge_get(annis_ASEdgePtr const* ptr, Edge item, AnnoKey key);

annis_Vec_Annotation annis_asedge_get_all(annis_ASEdgePtr const* ptr, Edge item);

size_t annis_asedge_guess_max_count(annis_ASEdgePtr const* ptr, annis_Option_StringID ns, StringID name, char const* lower_val, char const* upper_val);

size_t annis_asedge_guess_max_count_regex(annis_ASEdgePtr const* ptr, annis_Option_StringID ns, StringID name, char const* pattern);

void annis_asedge_calculate_statistics(annis_ASEdgePtr* ptr, annis_StringStoragePtr const* stringstorage);

annis_MatchIt* annis_asedge_exact_anno_search(annis_ASEdgePtr const* ptr, annis_Option_StringID ns, StringID name, annis_Option_StringID value);





#ifdef __cplusplus
}
#endif


#endif
