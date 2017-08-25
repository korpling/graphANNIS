
#ifndef cheddar_generated_graphannisapi_h
#define cheddar_generated_graphannisapi_h


#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stdbool.h>



typedef struct StringStoragePtr StringStoragePtr;

StringStoragePtr* annis_stringstorage_new(void);

void annis_stringstorage_free(StringStoragePtr* target);

typedef struct OptionalString {
	int valid;
	char const* value;
	size_t length;
} OptionalString;

OptionalString annis_stringstorage_str(StringStoragePtr const* target, uint32_t id);

uint32_t annis_stringstorage_add(StringStoragePtr* target, char const* value);

void annis_stringstorage_clear(StringStoragePtr* target);

size_t annis_stringstorage_len(StringStoragePtr const* target);



#ifdef __cplusplus
}
#endif


#endif
