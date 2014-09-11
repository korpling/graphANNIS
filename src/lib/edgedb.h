#ifndef EDGEDB_H
#define EDGEDB_H

#include <stdlib.h>
#include <cstdint>
#include <string>

#include "types.h"

namespace annis
{

class EdgeDB
{
public:
  virtual void addEdge(Edge edge) = 0;
  virtual void addEdgeAnnotation(Edge edge, const Annotation& anno) = 0;
  virtual void clear() = 0;

  virtual bool isConnected(const Edge& edge, unsigned int distance = 1) = 0;
  virtual std::vector<Annotation> getEdgeAnnotations(Edge edge) = 0;

  virtual std::string getName() = 0;
  virtual const Component& getComponent() = 0;
};
} // end namespace annis
#endif // EDGEDB_H
