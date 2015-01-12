#ifndef FALLBACKEDGEDB_H
#define FALLBACKEDGEDB_H

#include <stx/btree_map>
#include <stx/btree_multimap>
#include <stx/btree_set>
#include "../edgedb.h"
#include "../db.h"
#include "../comparefunctions.h"

#include <stack>
#include <set>

namespace annis
{


class FallbackEdgeDB : public EdgeDB
{
friend class FallbackDFSIterator;
friend class FallbackDFS;

public:
  FallbackEdgeDB(StringStorage& strings, const Component& component);

  virtual void addEdge(const Edge& edge);
  virtual void addEdgeAnnotation(const Edge &edge, const Annotation& anno);
  virtual void clear();

  virtual bool isConnected(const Edge& edge, unsigned int minDistance, unsigned int maxDistance) const;
  virtual std::unique_ptr<EdgeIterator> findConnected(nodeid_t sourceNode,
                                           unsigned int minDistance = 1,
                                           unsigned int maxDistance = 1) const;

  virtual int distance(const Edge &edge) const;

  virtual std::vector<Annotation> getEdgeAnnotations(const Edge &edge) const;
  virtual std::vector<nodeid_t> getOutgoingEdges(nodeid_t node) const;
  virtual std::vector<nodeid_t> getIncomingEdges(nodeid_t node) const;

  stx::btree_set<Edge, compEdges>::const_iterator getEdgesBegin()
  {
    return edges.begin();
  }
  stx::btree_set<Edge, compEdges>::const_iterator getEdgesEnd()
  {
    return edges.end();
  }

  virtual bool load(std::string dirPath);
  virtual bool save(std::string dirPath);

  virtual std::uint32_t numberOfEdges() const;
  virtual std::uint32_t numberOfEdgeAnnotations() const;

private:
  StringStorage& strings;
  Component component;

  stx::btree_set<Edge, compEdges> edges;
  stx::btree_multimap<Edge, Annotation, compEdges> edgeAnnotations;

};

struct DFSIteratorResult
{
  bool found;
  unsigned int distance;
  nodeid_t node;
};

/** A depth first traverser */
class FallbackDFSIterator : public EdgeIterator
{
public:

  FallbackDFSIterator(const FallbackEdgeDB& edb, std::uint32_t startNode, unsigned int minDistance, unsigned int maxDistance);

  virtual DFSIteratorResult nextDFS();
  virtual std::pair<bool, nodeid_t> next();

  virtual void reset();
private:

  const FallbackEdgeDB& edb;

  /**
   * @brief Traversion stack
   * Contains both the node id (first) and the distance from the start node (second)
   */
  std::stack<std::pair<nodeid_t, unsigned int> > traversalStack;
  unsigned int minDistance;
  unsigned int maxDistance;
  std::uint32_t startNode;

  std::set<nodeid_t> visited;
};

} // end namespace annis
#endif // FALLBACKEDGEDB_H
