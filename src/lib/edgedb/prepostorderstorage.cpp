#include "prepostorderstorage.h"

#include <set>
#include <stack>

#include <fstream>
#include <boost/archive/binary_oarchive.hpp>
#include <boost/archive/binary_iarchive.hpp>
#include <boost/serialization/map.hpp>


using namespace annis;

PrePostOrderStorage::PrePostOrderStorage(StringStorage &strings, const Component &component)
 : FallbackEdgeDB(strings, component)
{

}

PrePostOrderStorage::~PrePostOrderStorage()
{

}

bool PrePostOrderStorage::load(std::string dirPath)
{
  node2order.clear();
  preorder2node.clear();

  bool result = FallbackEdgeDB::load(dirPath);
  std::ifstream in;

  in.open(dirPath + "/node2order.btree", std::ios::binary);
  result = result && node2order.restore(in);
  in.close();

  in.open(dirPath + "/preorder2node.btree", std::ios::binary);
  result = result && preorder2node.restore(in);
  in.close();

  return result;
}

bool PrePostOrderStorage::save(std::string dirPath)
{
  bool result = FallbackEdgeDB::save(dirPath);

  std::ofstream out;

  out.open(dirPath + "/node2order.btree", std::ios::binary);
  node2order.dump(out);
  out.close();

  out.open(dirPath + "/preorder2node.btree", std::ios::binary);
  preorder2node.dump(out);
  out.close();

  return result;

}

void PrePostOrderStorage::calculateIndex()
{
  using ItType = stx::btree_set<Edge>::const_iterator;
  node2order.clear();
  preorder2node.clear();

  // find all roots of the component
  std::set<nodeid_t> roots;
  // first add all nodes that are a source of an edge as possible roots
  for(ItType it = getEdgesBegin(); it != getEdgesEnd(); it++)
  {
    roots.insert(it->source);
  }
  // second delete the ones that have an outgoing edge
  for(ItType it = getEdgesBegin(); it != getEdgesEnd(); it++)
  {
    roots.erase(it->target);
  }
  uint32_t currentOrder = 0;

  // traverse the graph for each sub-component
  for(const auto& startNode : roots)
  {
    unsigned int lastDistance = 0;

    std::stack<NodeStackEntry> nodeStack;

    enterNode(currentOrder, startNode, startNode, 0, nodeStack);

    FallbackDFSIterator dfs(*this, startNode, 1, uintmax);
    for(DFSIteratorResult step = dfs.nextDFS(); step.found;
          step = dfs.nextDFS())
    {
      if(step.distance > lastDistance)
      {
        // first visited, set pre-order
        enterNode(currentOrder, step.node, startNode, step.distance, nodeStack);
      }
      else
      {
        // Neighbour node, the last subtree was iterated completly, thus the last node
        // can be assigned a post-order.
        // The parent node must be at the top of the node stack,
        // thus exit every node which comes after the parent node.
        // Distance starts with 0 but the stack size starts with 1.
        while(nodeStack.size() > step.distance)
        {
          exitNode(currentOrder, nodeStack);
        }
        // new node
        enterNode(currentOrder, step.node, startNode, step.distance, nodeStack);
      }
      lastDistance = step.distance;
    } // end for each DFS step

    while(!nodeStack.empty())
    {
      exitNode(currentOrder, nodeStack);
    }

  } // end for each root
}

void PrePostOrderStorage::enterNode(uint32_t& currentOrder, nodeid_t nodeID, nodeid_t rootNode,
                                        int level, std::stack<NodeStackEntry>& nodeStack)
{
  preorder2node[currentOrder] = nodeID;
  NodeStackEntry newEntry;
  newEntry.id = nodeID;
  newEntry.order.pre = currentOrder++;
  newEntry.order.level = level;

  nodeStack.push(newEntry);
}

void PrePostOrderStorage::exitNode(uint32_t& currentOrder, std::stack<NodeStackEntry> &nodeStack)
{
  // find the correct pre/post entry and update the post-value
  auto& entry = nodeStack.top();
  entry.order.post = currentOrder++;

  node2order.insert2(entry.id, entry.order);

  nodeStack.pop();
}


bool PrePostOrderStorage::isConnected(const Edge &edge, unsigned int minDistance, unsigned int maxDistance) const
{

  const auto itSourceBegin = node2order.lower_bound(edge.source);
  const auto itSourceEnd = node2order.upper_bound(edge.source);

  for(auto itSource=itSourceBegin; itSource != itSourceEnd; itSource++)
  {
    auto itTargetRange = node2order.equal_range(edge.target);
    for(auto itTarget=itTargetRange.first; itTarget != itTargetRange.second; itTarget++)
    {
      if(itSource->second.pre <= itTarget->second.pre
         && itTarget->second.post <= itSource->second.post)
      {
        // check the level
        int diffLevel = (itTarget->second.level - itSource->second.level);
        if(minDistance <= diffLevel && diffLevel <= maxDistance)
        {
          return true;
        }
      }
    }
  }
  return false;
}

int PrePostOrderStorage::distance(const Edge &edge) const
{
  const auto itSourceBegin = node2order.lower_bound(edge.source);
  const auto itSourceEnd = node2order.upper_bound(edge.source);

  bool wasFound = false;
  int32_t minLevel = std::numeric_limits<int32_t>::max();

  for(auto itSource=itSourceBegin; itSource != itSourceEnd; itSource++)
  {
    auto itTargetRange = node2order.equal_range(edge.target);
    for(auto itTarget=itTargetRange.first; itTarget != itTargetRange.second; itTarget++)
    {
      if(itSource->second.pre <= itTarget->second.pre
         && itTarget->second.post <= itSource->second.post)
      {
        // check the level
        int32_t diffLevel = (itTarget->second.level - itSource->second.level);
        if(diffLevel >= 0)
        {
          wasFound = true;
          minLevel = std::min(minLevel, diffLevel);
        }
      }
    }
  }
  if(wasFound)
  {
    return minLevel;
  }
  else
  {
    return -1;
  }
}

std::unique_ptr<EdgeIterator> PrePostOrderStorage::findConnected(nodeid_t sourceNode, unsigned int minDistance, unsigned int maxDistance) const
{
  return std::unique_ptr<EdgeIterator>(
        new PrePostIterator(*this, sourceNode, minDistance, maxDistance));
}



PrePostIterator::PrePostIterator(const PrePostOrderStorage &storage, std::uint32_t startNode, unsigned int minDistance, unsigned int maxDistance)
  : storage(storage), startNode(startNode),
    minDistance(minDistance), maxDistance(maxDistance)
{
  reset();
}

std::pair<bool, nodeid_t> PrePostIterator::next()
{
  std::pair<bool, nodeid_t> result(0, false);

  while(!ranges.empty())
  {
    const auto& upper = ranges.top().upper;
    const auto& maximumPost = ranges.top().maximumPost;
    const auto& startLevel = ranges.top().startLevel;

    while(currentNode != upper)
    {
      const auto& currentOrderIt = storage.node2order.find(currentNode->second);
      if(currentOrderIt != storage.node2order.end())
      {
        const auto& currentPre = currentOrderIt->second.pre;
        const auto& currentPost = currentOrderIt->second.post;
        const auto& currentLevel = currentOrderIt->second.level;

        int diffLevel = currentLevel - startLevel;

        // check post order and level as well
        if(currentPost < maximumPost && minDistance <= diffLevel && diffLevel <= maxDistance)
        {
          // success
          result.first = true;
          result.second = currentNode->second;
          currentNode++;
          return result;
        }
        else if(currentPre < maximumPost)
        {
          // proceed with the next entry in the range
          currentNode++;
        }
        else
        {
          // abort searching in this range
          break;
        }

      } // end if the current pre-order could be mapped to a node
      else
      {
        currentNode++;
      }

    } // end while range not finished yet

    // this range is finished, try next one
    ranges.pop();
    if(!ranges.empty())
    {
      currentNode = ranges.top().lower;
    }
  }

  return result;
}

void PrePostIterator::reset()
{
  while(!ranges.empty())
  {
    ranges.pop();
  }

  auto subComponentsLower = storage.node2order.lower_bound(startNode);
  auto subComponentsUpper = storage.node2order.upper_bound(startNode);

  for(auto it=subComponentsLower; it != subComponentsUpper; it++)
  {
    auto pre = it->second.pre;
    auto post = it->second.post;
    auto lowerIt = storage.preorder2node.lower_bound(pre);
    auto upperIt = storage.preorder2node.upper_bound(post);

    ranges.push({lowerIt, upperIt, post, it->second.level});
  }

  if(!ranges.empty())
  {
    currentNode = ranges.top().lower;
  }

}

PrePostIterator::~PrePostIterator()
{

}
