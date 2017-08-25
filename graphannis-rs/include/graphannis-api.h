
#ifndef cheddar_generated_graphannisapi_h
#define cheddar_generated_graphannisapi_h


#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stdbool.h>



typedef struct annis_StringStoragePtr annis_StringStoragePtr;

typedef struct annis_OptionalString {
	int valid;
	char const* value;
	size_t length;
} annis_OptionalString;

typedef struct annis_Option_u32 {
	int valid;
	uint32_t value;
} annis_Option_u32;

annis_StringStoragePtr* annis_stringstorage_new(void);

void annis_stringstorage_free(annis_StringStoragePtr* target);

annis_OptionalString annis_stringstorage_str(annis_StringStoragePtr const* target, uint32_t id);

annis_Option_u32 annis_stringstorage_find_id(annis_StringStoragePtr const* target, char const* value);

uint32_t annis_stringstorage_add(annis_StringStoragePtr* target, char const* value);

void annis_stringstorage_clear(annis_StringStoragePtr* target);

size_t annis_stringstorage_len(annis_StringStoragePtr const* target);

double annis_stringstorage_avg_length(annis_StringStoragePtr const* target);

void annis_stringstorage_save_to_file(annis_StringStoragePtr const* target, char const* path);

void annis_stringstorage_load_from_file(annis_StringStoragePtr* target, char const* path);

size_t annis_stringstorage_estimate_memory(annis_StringStoragePtr const* target);



#ifdef __cplusplus
}
#endif


#endif
