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
  order2node[currentOrder] = nodeID;
  PrePost newEntry;
  newEntry.pre = currentOrder++;
  newEntry.level = level;
  Node n;
  n.id = nodeID;
  n.root = rootNode;
  node2order.insert2(n, newEntry);
  nodeStack.push(nodeID);
}

void PrePostOrderStorage::exitNode(uint32_t& currentOrder, std::stack<nodeid_t>& nodeStack, uint32_t rootNode)
{
  order2node[currentOrder] = nodeStack.top();
  // find the correct pre/post entry and update the post-value
  Node n;
  n.id = nodeStack.top();
  n.root = rootNode;
  node2order[n].post = currentOrder++;
  nodeStack.pop();
}


bool PrePostOrderStorage::isConnected(const Edge &edge, unsigned int minDistance, unsigned int maxDistance)
{
  Node sourceLower;
  sourceLower.id = edge.source;
  sourceLower.root = 0;

  Node sourceUpper;
  sourceUpper.id = edge.source;
  sourceUpper.root = uintmax;

  const auto itSourceBegin = node2order.lower_bound(sourceLower);
  const auto itSourceEnd = node2order.upper_bound(sourceUpper);

  for(auto itSource=itSourceBegin; itSource != itSourceEnd; itSource++)
  {
    Node target;
    target.id = edge.target;
    target.root = itSource->first.root;

    const auto itTarget = node2order.find(target);
    if(itTarget != node2order.end())
    {
      if(itSource->second.pre <= itTarget->second.pre
         && itTarget->second.post <= itSource->second.post
         && itSource->first.root && itTarget->first.root)
      {
        // check the level
        int32_t diffLevel = (itTarget->second.level - itSource->second.level);
        if(minDistance <= diffLevel <= maxDistance)
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
  Node sourceLower;
  sourceLower.id = edge.source;
  sourceLower.root = 0;

  Node sourceUpper;
  sourceUpper.id = edge.source;
  sourceUpper.root = uintmax;

  const auto itSourceBegin = node2order.lower_bound(sourceLower);
  const auto itSourceEnd = node2order.upper_bound(sourceUpper);

  for(auto itSource=itSourceBegin; itSource != itSourceEnd; itSource++)
  {
    Node target;
    target.id = edge.target;
    target.root = itSource->first.root;

    const auto itTarget = node2order.find(target);
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

