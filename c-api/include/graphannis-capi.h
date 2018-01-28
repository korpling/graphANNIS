
#ifndef cheddar_generated_annis_graphanniscapi_h
#define cheddar_generated_annis_graphanniscapi_h


#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stdbool.h>



typedef struct annis_CorpusStorage annis_CorpusStorage;

/// Create a new corpus storage
annis_CorpusStorage* annis_cs_new(char const* db_dir);

/// Delete a corpus storage
void annis_cs_free(annis_CorpusStorage* ptr);

uint64_t annis_cs_count(annis_CorpusStorage const* ptr, char const* query_as_json, char const* corpus);





#ifdef __cplusplus
}
#endif


#endif
