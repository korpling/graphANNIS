#ifndef EDGEDB_H
#define EDGEDB_H

#include <stdlib.h>
#include <cstdint>
#include <string>
#include <vector>

#include "types.h"
#include "annotationiterator.h"
#include "stringstorage.h"

namespace annis
{

class EdgeDB
{
public:
  virtual void addEdge(const Edge& edge) = 0;
  virtual void addEdgeAnnotation(const Edge& edge, const Annotation& anno) = 0;
  virtual void clear() = 0;
  virtual void calculateIndex() {}

  virtual bool isConnected(const Edge& edge, unsigned int minDistance = 1, unsigned int maxDistance = 1) const = 0;
  /**
   * @brief Returns a newly allocated iterator for the connected nodes.
   * @param sourceNode
   * @param minDistance
   * @param maxDistance
   * @return An iterator. You have to delete this iterator by yourself after using it.
   */
  virtual EdgeIterator* findConnected(
                                           nodeid_t sourceNode,
                                           unsigned int minDistance = 1,
                                           unsigned int maxDistance = 1) const = 0;

  virtual std::vector<Annotation> getEdgeAnnotations(const Edge& edge) const = 0;
  virtual std::vector<nodeid_t> getOutgoingEdges(nodeid_t node) const = 0;
  virtual std::vector<nodeid_t> getIncomingEdges(nodeid_t node) const = 0;

  virtual bool load(std::string dirPath) = 0;
  virtual bool save(std::string dirPath) = 0;

  virtual std::uint32_t numberOfEdges() const = 0;
  virtual std::uint32_t numberOfEdgeAnnotations() const = 0;
};
} // end namespace annis
#endif // EDGEDB_H
