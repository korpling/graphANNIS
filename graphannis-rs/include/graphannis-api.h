
#ifndef cheddar_generated_graphannisapi_h
#define cheddar_generated_graphannisapi_h


#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stdbool.h>



typedef struct annis_StringStoragePtr annis_StringStoragePtr;

annis_StringStoragePtr* annis_stringstorage_new(void);

void annis_stringstorage_free(annis_StringStoragePtr* target);

typedef struct annis_OptionalString {
	int valid;
	char const* value;
	size_t length;
} annis_OptionalString;

annis_OptionalString annis_stringstorage_str(annis_StringStoragePtr const* target, uint32_t id);

uint32_t annis_stringstorage_add(annis_StringStoragePtr* target, char const* value);

void annis_stringstorage_clear(annis_StringStoragePtr* target);

size_t annis_stringstorage_len(annis_StringStoragePtr const* target);



#ifdef __cplusplus
}
#endif


#endif
