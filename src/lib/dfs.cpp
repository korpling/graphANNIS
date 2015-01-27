#include "dfs.h"

#include <iostream>

using namespace annis;

DFS::DFS(const ReadableGraphStorage &edb,
                                                     std::uint32_t startNode,
                                                     unsigned int minDistance,
                                                     unsigned int maxDistance)
  : edb(edb), minDistance(minDistance), maxDistance(maxDistance), startNode(startNode)
{
  // add the initial value to the stack
  traversalStack.push({startNode, 0});
}

DFSIteratorResult DFS::nextDFS()
{
  DFSIteratorResult result;
  result.found = false;

  while(!result.found && !traversalStack.empty())
  {
    std::pair<uint32_t, unsigned int> stackEntry = traversalStack.top();
    result.node = stackEntry.first;
    result.distance = stackEntry.second;


    // we are entering a new node
    if(beforeEnterNode(result.node, result.distance))
    {
      result.found = enterNode(result.node, result.distance);
    }
    else
    {
      traversalStack.pop();
    }
  }
  return result;
}

bool DFS::enterNode(nodeid_t node, unsigned int distance)
{
  bool found = false;

  traversalStack.pop();

  if(distance >= minDistance && distance <= maxDistance)
  {
    // get the next node
    found = true;
  }

  // add the remaining child nodes
  if(distance < maxDistance)
  {
    // add the outgoing edges to the stack
    auto outgoing = edb.getOutgoingEdges(node);
    for(const auto& outNodeID : outgoing)
    {

      traversalStack.push(std::pair<nodeid_t, unsigned int>(outNodeID, distance+1));
    }
  }
  return found;
}


std::pair<bool, nodeid_t> DFS::next()
{
  DFSIteratorResult result = nextDFS();
  return std::pair<bool, nodeid_t>(result.found, result.node);
}


void DFS::reset()
{
  // clear the stack
  while(!traversalStack.empty())
  {
    traversalStack.pop();
  }

  traversalStack.push({startNode, 0});
}


CycleSafeDFS::CycleSafeDFS(const ReadableGraphStorage &edb, std::uint32_t startNode, unsigned int minDistance, unsigned int maxDistance)
  : DFS(edb, startNode, minDistance, maxDistance), lastDistance(0)
{
  nodesInCurrentPath.insert(startNode);
  distanceToNode.insert({0, startNode});
}


void CycleSafeDFS::reset()
{
  nodesInCurrentPath.clear();
  distanceToNode.clear();

  DFS::reset();

  nodesInCurrentPath.insert(startNode);
  distanceToNode.insert({0, startNode});
}

bool CycleSafeDFS::enterNode(nodeid_t node, unsigned int distance)
{
  nodesInCurrentPath.insert(node);
  distanceToNode.insert({distance, node});

  lastDistance = distance;

  return DFS::enterNode(node, distance);
}

bool CycleSafeDFS::beforeEnterNode(nodeid_t node, unsigned int distance)
{
  if(lastDistance >= distance)
  {
    // A subgraph was completed.
    // Remove all nodes from the path set that are below the parent node:
    for(auto it=distanceToNode.find(distance); it != distanceToNode.end(); it = distanceToNode.erase(it))
    {
      nodesInCurrentPath.erase(it->second);
    }
  }

  if(nodesInCurrentPath.find(node) == nodesInCurrentPath.end())
  {
    return true;
  }
  else
  {
    // we detected a cycle!
    std::cerr << "------------------------------" << std::endl;
    std::cerr << "ERROR: cycle detected when inserting node " << node << std::endl;
    std::cerr << "distanceToNode: ";
    for(auto itPath = distanceToNode.begin(); itPath != distanceToNode.end(); itPath++)
    {
      std::cerr << itPath->first << "->" << itPath->second << " ";
    }
    std::cerr << std::endl;
    std::cerr << "nodesInCurrentPath: ";
    for(auto itPath = nodesInCurrentPath.begin(); itPath != nodesInCurrentPath.end(); itPath++)
    {
      std::cerr << *itPath << " ";
    }
    std::cerr << std::endl;
    std::cerr << "------------------------------" << std::endl;

    lastDistance = distance;

    return false;
  }
}

CycleSafeDFS::~CycleSafeDFS()
{

}


UniqueDFS::UniqueDFS(const ReadableGraphStorage &edb, std::uint32_t startNode, unsigned int minDistance, unsigned int maxDistance)
  : DFS(edb, startNode, minDistance, maxDistance)
{

}

UniqueDFS::~UniqueDFS()
{

}

void UniqueDFS::reset()
{
  DFS::reset();
  visited.clear();
}

bool UniqueDFS::enterNode(nodeid_t node, unsigned int distance)
{
  visited.insert(node);
  return DFS::enterNode(node, distance);
}

bool UniqueDFS::beforeEnterNode(nodeid_t node, unsigned int distance)
{
  return visited.find(node) == visited.end();
}
