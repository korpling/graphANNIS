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
  virtual void addEdge(const Edge& edge) = 0;
  virtual void addEdgeAnnotation(const Edge& edge, const Annotation& anno) = 0;
  virtual void clear() = 0;

  virtual bool isConnected(const Edge& edge, unsigned int distance = 1) = 0;
  virtual std::vector<Annotation> getEdgeAnnotations(const Edge& edge) = 0;

  virtual std::string getName() = 0;
  virtual const Component& getComponent() = 0;

  virtual bool load(std::string dirPath) = 0;
  virtual bool save(std::string dirPath) = 0;

  virtual std::uint32_t numberOfEdges() const = 0;
  virtual std::uint32_t numberOfEdgeAnnotations() const = 0;
};
} // end namespace annis
#endif // EDGEDB_H
