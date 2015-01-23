#ifndef DFS_H
#define DFS_H

#include "edgedb.h"

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

  DFS(const EdgeDB& edb,
      std::uint32_t startNode,
      unsigned int minDistance, unsigned int maxDistance);

  virtual DFSIteratorResult nextDFS();
  virtual std::pair<bool, nodeid_t> next();

  void reset();
  virtual ~DFS() {}
protected:
  const nodeid_t startNode;


  void initStack();

  virtual bool enterNode(nodeid_t node, unsigned int distance);

  virtual bool beforeEnterNode(nodeid_t node, unsigned int distance)
  {
    return true;
  }

private:

  const EdgeDB& edb;

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

  UniqueDFS(const EdgeDB& edb,
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

  CycleSafeDFS(const EdgeDB& edb,
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


#endif // DFS_H
