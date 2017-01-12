/*
   Copyright 2017 Thomas Krause <thomaskrause@posteo.de>

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

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
  virtual std::pair<bool, nodeid_t> next() override;

  void reset() override;
  virtual ~DFS() {}
protected:
  const nodeid_t startNode;

  virtual bool enterNode(nodeid_t node, unsigned int distance);

  virtual bool beforeEnterNode(nodeid_t /* node */, unsigned int /* distance */)
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
  virtual void reset() override;
  virtual bool enterNode(nodeid_t node, unsigned int distance) override;
  virtual bool beforeEnterNode(nodeid_t node, unsigned int distance) override;


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
  virtual void reset() override;
  virtual bool enterNode(nodeid_t node, unsigned int distance) override;
  virtual bool beforeEnterNode(nodeid_t node, unsigned int distance) override;


private:
  unsigned int lastDistance;
  std::set<nodeid_t> nodesInCurrentPath;
  std::multimap<unsigned int, nodeid_t> distanceToNode;
  bool outputCycleErrors;
  bool cycleDetected;
};

} // end namespace annis


