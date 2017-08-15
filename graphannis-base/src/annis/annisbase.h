#ifndef _ANNIS_BASE_LIB_HEADER_H_
#define _ANNIS_BASE_LIB_HEADER_H_

#ifdef __cplusplus
extern "C" {
#endif

void* annis_stringstorage_new();
char* annis_stringstorage_str(void* strstor, long long id);

#ifdef __cplusplus
}
#endif

#endif //_ANNIS_BASE_LIB_HEADER_H_

