#ifndef LINEAREDGEDB_H
#define LINEAREDGEDB_H

#include <stx/btree_map>
#include "../edgedb.h"

namespace annis
{
class LinearEdgeDB : public EdgeDB
{
public:
  LinearEdgeDB();

  virtual void addEdge(const Edge& edge);
  virtual void addEdgeAnnotation(const Edge& edge, const Annotation& anno);
  virtual void clear();

  virtual bool isConnected(const Edge& edge, unsigned int minDistance, unsigned int maxDistance) const;
  virtual EdgeIterator* findConnected(
                                           nodeid_t sourceNode,
                                           unsigned int minDistance,
                                           unsigned int maxDistance) const;

  virtual std::vector<Annotation> getEdgeAnnotations(const Edge& edge) const;
  virtual std::vector<nodeid_t> getOutgoingEdges(nodeid_t sourceNode) const;

  virtual bool load(std::string dirPath) = 0;
  virtual bool save(std::string dirPath) = 0;

  virtual std::uint32_t numberOfEdges() const = 0;
  virtual std::uint32_t numberOfEdgeAnnotations() const = 0;

  virtual ~LinearEdgeDB();

private:
  stx::btree_map<nodeid_t, std::uint32_t> node2pos;
  stx::btree_map<std::uint32_t, nodeid_t> pos2node;
};

} // end namespace annis

#endif // LINEAREDGEDB_H
