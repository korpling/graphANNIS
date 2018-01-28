
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





#ifdef __cplusplus
}
#endif


#endif
