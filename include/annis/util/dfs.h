#pragma once

#include <annis/graphstorage/graphstorage.h>

#include <stack>
#include <list>

namespace annis
{

struct DFSIteratorResult
{
  bool found;
  unsigned int distance;
  nodeid_t node;
};


/** A depth first traverser */
class DFS : public EdgeIterator
{
public:

  DFS(const ReadableGraphStorage& gs,
      std::uint32_t startNode,
      unsigned int minDistance, unsigned int maxDistance);

  virtual DFSIteratorResult nextDFS();
  virtual std::pair<bool, nodeid_t> next();

  void reset();
  virtual ~DFS() {}
protected:
  const nodeid_t startNode;

  virtual bool enterNode(nodeid_t node, unsigned int distance);

  virtual bool beforeEnterNode(nodeid_t node, unsigned int distance)
  {
    return true;
  }

private:

  const ReadableGraphStorage& gs;

  using TraversalEntry = std::pair<nodeid_t, unsigned int>;

  /**
   * @brief Traversion stack
   * Contains both the node id (first) and the distance from the start node (second)
   */
  std::stack<TraversalEntry, std::list<TraversalEntry> > traversalStack;
  unsigned int minDistance;
  unsigned int maxDistance;

};

/**
 * @brief Traverses a graph and visits any node at maximum once.
 */
class UniqueDFS : public DFS
{
public:

  UniqueDFS(const ReadableGraphStorage& gs,
               std::uint32_t startNode,
               unsigned int minDistance, unsigned int maxDistance);
  virtual ~UniqueDFS();
protected:
  virtual void reset();
  virtual bool enterNode(nodeid_t node, unsigned int distance);
  virtual bool beforeEnterNode(nodeid_t node, unsigned int distance);


private:

  std::set<nodeid_t> visited;
};


/**
 * @brief A cycle safe implementation of depth first traversal
 */
class CycleSafeDFS : public DFS
{
public:

  CycleSafeDFS(const ReadableGraphStorage& gs,
               std::uint32_t startNode,
               unsigned int minDistance, unsigned int maxDistance,
               bool outputCycleErrors = true);
  virtual ~CycleSafeDFS();

  virtual bool cyclic()
  {
    return cycleDetected;
  }

protected:
  virtual void reset();
  virtual bool enterNode(nodeid_t node, unsigned int distance);
  virtual bool beforeEnterNode(nodeid_t node, unsigned int distance);


private:
  unsigned int lastDistance;
  std::set<nodeid_t> nodesInCurrentPath;
  std::multimap<unsigned int, nodeid_t> distanceToNode;
  bool outputCycleErrors;
  bool cycleDetected;
};

} // end namespace annis


