
#ifndef cheddar_generated_annis_graphanniscapi_h
#define cheddar_generated_annis_graphanniscapi_h


#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stdbool.h>



typedef uint32_t NodeID;

typedef uint32_t StringID;

typedef struct AnnoKey {
	StringID name;
	StringID ns;
} AnnoKey;

typedef struct Annotation {
	AnnoKey key;
	StringID val;
} Annotation;

typedef struct Edge {
	NodeID source;
	NodeID target;
} Edge;



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

typedef struct annis_Vec_Annotation {
	Annotation const* v;
	size_t length;
} annis_Vec_Annotation;



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

annis_Option_u32 annis_asnode_remove(annis_ASNodePtr* ptr, NodeID item, AnnoKey key);

size_t annis_asnode_len(annis_ASNodePtr const* ptr);

annis_Option_u32 annis_asnode_get(annis_ASNodePtr const* ptr, NodeID item, AnnoKey key);

annis_Vec_Annotation annis_asnode_get_all(annis_ASNodePtr const* ptr, NodeID item);

annis_ASEdgePtr* annis_asedge_new(void);

void annis_asedge_free(annis_ASEdgePtr* ptr);

void annis_asedge_insert(annis_ASEdgePtr* ptr, Edge item, Annotation anno);

annis_Option_u32 annis_asedge_remove(annis_ASEdgePtr* ptr, Edge item, AnnoKey key);

size_t annis_asedge_len(annis_ASEdgePtr const* ptr);

annis_Option_u32 annis_asedge_get(annis_ASEdgePtr const* ptr, Edge item, AnnoKey key);

annis_Vec_Annotation annis_asedge_get_all(annis_ASEdgePtr const* ptr, Edge item);





#ifdef __cplusplus
}
#endif


#endif
