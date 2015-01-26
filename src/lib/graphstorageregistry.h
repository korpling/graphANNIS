#ifndef GRAPHSTORAGEREGISTRY_H
#define GRAPHSTORAGEREGISTRY_H

#include "edgedb.h"

namespace annis
{

class GraphStorageRegistry
{
public:
  GraphStorageRegistry();
  ~GraphStorageRegistry();

  std::string getName(const EdgeDB* db);
  EdgeDB* createEdgeDB(std::string name, StringStorage &strings, const Component &component);

};

}

#endif // GRAPHSTORAGEREGISTRY_H
