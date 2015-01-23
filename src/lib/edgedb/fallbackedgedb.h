#ifndef FALLBACKEDGEDB_H
#define FALLBACKEDGEDB_H

#include <stx/btree_map>
#include <stx/btree_multimap>
#include <stx/btree_set>
#include "../edgedb.h"
#include "../db.h"
#include "../comparefunctions.h"

#include <stack>
#include <list>
#include <set>
#include <map>

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

  stx::btree_set<Edge>::const_iterator getEdgesBegin()
  {
    return edges.begin();
  }
  stx::btree_set<Edge>::const_iterator getEdgesEnd()
  {
    return edges.end();
  }

  virtual bool load(std::string dirPath);
  virtual bool save(std::string dirPath);

  virtual std::uint32_t numberOfEdges() const;
  virtual std::uint32_t numberOfEdgeAnnotations() const;
  const Component& getComponent() { return component;}

private:
  StringStorage& strings;
  Component component;

  stx::btree_set<Edge> edges;
  stx::btree_multimap<Edge, Annotation> edgeAnnotations;

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

  FallbackDFSIterator(const FallbackEdgeDB& edb,
                      std::uint32_t startNode,
                      unsigned int minDistance, unsigned int maxDistance);

  virtual DFSIteratorResult nextDFS();
  virtual std::pair<bool, nodeid_t> next();

  void reset();
  virtual ~FallbackDFSIterator() {}
protected:
  const nodeid_t startNode;


  void initStack();

  virtual bool enterNode(nodeid_t node, unsigned int distance);

  virtual bool beforeEnterNode(nodeid_t node, unsigned int distance)
  {
    return true;
  }

private:

  const FallbackEdgeDB& edb;

  using TraversalEntry = std::pair<nodeid_t, unsigned int>;

  /**
   * @brief Traversion stack
   * Contains both the node id (first) and the distance from the start node (second)
   */
  std::stack<TraversalEntry, std::list<TraversalEntry> > traversalStack;
  unsigned int minDistance;
  unsigned int maxDistance;

};

class CycleSafeDFS : public FallbackDFSIterator
{
public:

  CycleSafeDFS(const FallbackEdgeDB& edb,
                      std::uint32_t startNode,
                      unsigned int minDistance, unsigned int maxDistance);
  virtual ~CycleSafeDFS();
protected:
  virtual void initStack();
  virtual void reset();
  virtual bool enterNode(nodeid_t node, unsigned int distance);
  virtual bool beforeEnterNode(nodeid_t node, unsigned int distance);


private:
  unsigned int lastDistance;
  std::set<nodeid_t> nodesInCurrentPath;
  std::multimap<unsigned int, nodeid_t> distanceToNode;
};


} // end namespace annis
#endif // FALLBACKEDGEDB_H
