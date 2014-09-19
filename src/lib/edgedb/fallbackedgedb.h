#ifndef FALLBACKEDGEDB_H
#define FALLBACKEDGEDB_H

#include <stx/btree_map>
#include <stx/btree_multimap>
#include "../edgedb.h"
#include "../db.h"
#include "../comparefunctions.h"

#include <stack>

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

  virtual std::string getName() {return "fallback";}
  virtual const Component& getComponent();

  virtual bool isConnected(const Edge& edge, unsigned int minDistance, unsigned int maxDistance) const;
  virtual EdgeIterator* findConnected(std::uint32_t sourceNode,
                                           unsigned int minDistance = 1,
                                           unsigned int maxDistance = 1) const;
  virtual std::vector<Annotation> getEdgeAnnotations(const Edge &edge) const;
  virtual std::vector<std::uint32_t> getOutgoingEdges(std::uint32_t sourceNode) const;

  virtual bool load(std::string dirPath);
  virtual bool save(std::string dirPath);

  virtual std::uint32_t numberOfEdges() const;
  virtual std::uint32_t numberOfEdgeAnnotations() const;

private:
  StringStorage& strings;
  Component component;

  //TODO: it might be better to use a map of pair<uint32> -> bool
  stx::btree_multimap<std::uint32_t, std::uint32_t> edges;
  stx::btree_multimap<Edge, Annotation, compEdges> edgeAnnotations;


};


/** A depth first traverser */
class FallbackDFSIterator : public EdgeIterator
{
  typedef stx::btree_multimap<std::uint32_t, std::uint32_t>::const_iterator EdgeIt;

public:

  FallbackDFSIterator(const FallbackEdgeDB& edb, std::uint32_t startNode, unsigned int minDistance, unsigned int maxDistance);

  virtual std::pair<bool, std::uint32_t> next();
private:

  const FallbackEdgeDB& edb;

  /**
   * @brief Traversion stack
   * Contains both the node id (first) and the distance from the start node (second)
   */
  std::stack<std::pair<std::uint32_t, unsigned int> > traversalStack;
  unsigned int minDistance;
  unsigned int maxDistance;
};

} // end namespace annis
#endif // FALLBACKEDGEDB_H
