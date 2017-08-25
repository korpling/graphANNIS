#ifndef _ANNIS_BASE_LIB_HEADER_H_
#define _ANNIS_BASE_LIB_HEADER_H_


#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stdlib.h>


struct annis_OptionalString {
    int valid;
    /* not null-terminated string, use length to get the length of the string */
    const char* value;
    size_t length;
};

void* annis_stringstorage_new();
void annis_stringstorage_free(void* strstor);

uint32_t annis_stringstorage_add(void* strstor, const char* value);
annis_OptionalString annis_stringstorage_str(void* strstor, uint32_t id);


#ifdef __cplusplus
}
#endif

#endif //_ANNIS_BASE_LIB_HEADER_H_

