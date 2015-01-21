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
  order2node.clear();

  bool result = FallbackEdgeDB::load(dirPath);
  std::ifstream in;

  in.open(dirPath + "/node2order.btree", std::ios::binary);
  result = result && node2order.restore(in);
  in.close();

  in.open(dirPath + "/order2node.btree", std::ios::binary);
  result = result && order2node.restore(in);
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

  out.open(dirPath + "/order2node.btree", std::ios::binary);
  order2node.dump(out);
  out.close();

  return result;

}

void PrePostOrderStorage::calculateIndex()
{
  using ItType = stx::btree_set<Edge>::const_iterator;
  node2order.clear();
  order2node.clear();

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

  // traverse the graph for each sub-component
  for(const auto& startNode : roots)
  {
    unsigned int lastDistance = 0;

    uint32_t currentOrder = 0;
    std::stack<nodeid_t> nodeStack;

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
      else if(step.distance == lastDistance)
      {
        // neighbour node, the last subtree was iterated completly, thus the last node
        // can be assigned a post-order
        exitNode(currentOrder, nodeStack, startNode);

        // new node
        enterNode(currentOrder, step.node, startNode, step.distance, nodeStack);
      }
      else
      {
        // parent node, the subtree was iterated completly, thus the last node
        // can be assigned a post-order
        exitNode(currentOrder, nodeStack, startNode);

        // the current node was already visited
      }
      lastDistance = step.distance;
    } // end for each DFS step

    while(!nodeStack.empty())
    {
      exitNode(currentOrder, nodeStack, startNode);
    }

  } // end for each root
}

void PrePostOrderStorage::enterNode(uint32_t& currentOrder, nodeid_t nodeID, nodeid_t rootNode,
                                        int32_t level, std::stack<nodeid_t>& nodeStack)
{
  order2node[currentOrder] = {nodeID, rootNode};
  PrePost newEntry;
  newEntry.pre = currentOrder++;
  newEntry.level = level;
  node2order.insert2({nodeID, rootNode}, newEntry);
  nodeStack.push(nodeID);
}

void PrePostOrderStorage::exitNode(uint32_t& currentOrder, std::stack<nodeid_t>& nodeStack, nodeid_t rootNode)
{
  order2node[currentOrder] = {nodeStack.top(), rootNode};
  // find the correct pre/post entry and update the post-value
  Node n;
  n.id = nodeStack.top();
  n.root = rootNode;
  node2order[n].post = currentOrder++;
  nodeStack.pop();
}


bool PrePostOrderStorage::isConnected(const Edge &edge, unsigned int minDistance, unsigned int maxDistance) const
{

  const auto itSourceBegin = node2order.lower_bound({edge.source, 0});
  const auto itSourceEnd = node2order.upper_bound({edge.source, uintmax});

  for(auto itSource=itSourceBegin; itSource != itSourceEnd; itSource++)
  {
    const auto itTarget = node2order.find({edge.target, itSource->first.root});
    if(itTarget != node2order.end())
    {
      if(itSource->second.pre <= itTarget->second.pre
         && itTarget->second.post <= itSource->second.post
         && itSource->first.root && itTarget->first.root)
      {
        // check the level
        int32_t diffLevel = (itTarget->second.level - itSource->second.level);
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
  const auto itSourceBegin = node2order.lower_bound({ edge.source, 0 });
  const auto itSourceEnd = node2order.upper_bound({edge.source, uintmax});

  for(auto itSource=itSourceBegin; itSource != itSourceEnd; itSource++)
  {
    const auto itTarget = node2order.find({edge.target, itSource->first.root});
    if(itTarget != node2order.end())
    {
      if(itSource->second.pre <= itTarget->second.pre
         && itTarget->second.post <= itSource->second.post
         && itSource->first.root && itTarget->first.root)
      {
        // check the level
        int32_t diffLevel = (itTarget->second.level - itSource->second.level);
        return diffLevel;
      }
    }
  }
  return -1;
}

std::unique_ptr<EdgeIterator> PrePostOrderStorage::findConnected(nodeid_t sourceNode, unsigned int minDistance, unsigned int maxDistance) const
{
  return std::unique_ptr<EdgeIterator>(new PrePostIterator(*this, sourceNode, minDistance, maxDistance));
}



PrePostIterator::PrePostIterator(const PrePostOrderStorage &storage, std::uint32_t startNode, unsigned int minDistance, unsigned int maxDistance)
  : storage(storage), startNode(startNode), minDistance(minDistance), maxDistance(maxDistance)
{
  reset();
}

std::pair<bool, nodeid_t> PrePostIterator::next()
{
  std::pair<bool, nodeid_t> result(0, false);

  while(!ranges.empty())
  {
    while(currentNode != ranges.top().second)
    {

      // check post order
      const Node& maximumNode = ranges.top().second->second;
      const Node& candidateNode = currentNode->second;

      const PrePost& maximumOrder = (storage.node2order.find(maximumNode))->second;
      const PrePost& candidateOrder = (storage.node2order.find(candidateNode))->second;
      if(candidateOrder.post <= maximumOrder.post)
      {
        result.first = true;
        result.second = currentNode->second.id;
        currentNode++;
        return result;
      }
      else
      {
        currentNode++;
      }

    }

    // this range is finished, try next one
    ranges.pop();
    if(!ranges.empty())
    {
      currentNode = ranges.top().first;
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

  auto subComponentsLower = storage.node2order.lower_bound({startNode, 0});
  auto subComponentsUpper = storage.node2order.upper_bound({startNode, uintmax});

  for(auto it=subComponentsLower; it != subComponentsUpper; it++)
  {
    ranges.push(
          std::pair<OrderIt, OrderIt>(
            storage.order2node.lower_bound(it->second.pre),
            storage.order2node.upper_bound(it->second.post)));
  }

  if(!ranges.empty())
  {
    currentNode = ranges.top().first;
  }

}

PrePostIterator::~PrePostIterator()
{

}
