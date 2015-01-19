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

    order2node[currentOrder] = startNode;
    node2order[startNode].pre = currentOrder++;
    node2order[startNode].level = 0;
    nodeStack.push(startNode);

    FallbackDFSIterator dfs(*this, startNode, 1, uintmax);
    for(DFSIteratorResult step = dfs.nextDFS(); step.found;
          step = dfs.nextDFS())
    {
      if(step.distance > lastDistance)
      {
        // first visited, set pre-order
        order2node[currentOrder] = step.node;
        node2order[step.node].pre = currentOrder++;
        node2order[step.node].level = step.distance;
        nodeStack.push(step.node);
      }
      else if(step.distance == lastDistance)
      {
        // neighbour node, the last subtree was iterated completly, thus the last node
        // can be assigned a post-order
        order2node[currentOrder] = nodeStack.top();
        node2order[nodeStack.top()].post = currentOrder++;
        nodeStack.pop();

        // new node
        order2node[currentOrder] = step.node;
        node2order[step.node].pre = currentOrder++;
        node2order[step.node].level = step.distance;
        nodeStack.push(step.node);

      }
      else
      {
        // parent node, the subtree was iterated completly, thus the last node
        // can be assigned a post-order
        order2node[currentOrder] = nodeStack.top();
        node2order[nodeStack.top()].post = currentOrder++;
        nodeStack.pop();

        // the current node was already visited
      }
      lastDistance = step.distance;
    } // end for each DFS step
  } // end for each root
}

bool PrePostOrderStorage::isConnected(const Edge &edge, unsigned int minDistance, unsigned int maxDistance)
{
  const auto& orderSource = node2order.find(edge.source);
  const auto& orderTarget = node2order.find(edge.target);
  if(orderSource != node2order.end() && orderTarget != node2order.end())
  {
    if(orderSource->second.pre <= orderTarget->second.pre
       && orderTarget->second.post <= orderSource->second.post)
    {
      // check the level
      int32_t diffLevel = (orderTarget->second.level - orderSource->second.level);
      return minDistance <= diffLevel <= maxDistance;
    }
  }
  return false;
}

