
#ifndef cheddar_generated_annis_stringstorage_h
#define cheddar_generated_annis_stringstorage_h


#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stdbool.h>



typedef struct annis_StringStoragePtr annis_StringStoragePtr;

typedef struct annis_OptionalString {
	int valid;
	c_char const* value;
	size_t length;
} annis_OptionalString;

typedef struct annis_Option_u32 {
	int valid;
	uint32_t value;
} annis_Option_u32;

annis_StringStoragePtr* annis_stringstorage_new(void);

void annis_stringstorage_free(annis_StringStoragePtr* ptr);

annis_OptionalString annis_stringstorage_str(annis_StringStoragePtr const* ptr, uint32_t id);

annis_Option_u32 annis_stringstorage_find_id(annis_StringStoragePtr const* ptr, c_char const* value);

uint32_t annis_stringstorage_add(annis_StringStoragePtr* ptr, c_char const* value);

void annis_stringstorage_clear(annis_StringStoragePtr* ptr);

size_t annis_stringstorage_len(annis_StringStoragePtr const* ptr);

double annis_stringstorage_avg_length(annis_StringStoragePtr const* ptr);

void annis_stringstorage_save_to_file(annis_StringStoragePtr const* ptr, c_char const* path);

void annis_stringstorage_load_from_file(annis_StringStoragePtr* ptr, c_char const* path);

size_t annis_stringstorage_estimate_memory(annis_StringStoragePtr const* ptr);



#ifdef __cplusplus
}
#endif


#endif
