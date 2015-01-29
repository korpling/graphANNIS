#include "prepostorderstorage.h"

#include "../dfs.h"
#include "../exactannovaluesearch.h"
#include "../exactannokeysearch.h"

#include <set>
#include <stack>

#include <fstream>
#include <boost/archive/binary_oarchive.hpp>
#include <boost/archive/binary_iarchive.hpp>
#include <boost/serialization/map.hpp>


using namespace annis;

PrePostOrderStorage::PrePostOrderStorage(StringStorage &strings, const Component &component)
{

}

PrePostOrderStorage::~PrePostOrderStorage()
{

}

bool PrePostOrderStorage::load(std::string dirPath)
{
  node2order.clear();
  order2node.clear();

  bool result = edgeAnno.load(dirPath);
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
  bool result = edgeAnno.save(dirPath);

  std::ofstream out;

  out.open(dirPath + "/node2order.btree", std::ios::binary);
  node2order.dump(out);
  out.close();

  out.open(dirPath + "/order2node.btree", std::ios::binary);
  order2node.dump(out);
  out.close();

  return result;

}

void PrePostOrderStorage::copy(const DB& db, const ReadableGraphStorage& orig)
{
  using ItType = stx::btree_set<Edge>::const_iterator;
  clear();

  // find all roots of the component
  std::set<nodeid_t> roots;
  ExactAnnoKeySearch nodes(db, annis_ns, annis_node_name);
  // first add all nodes that are a source of an edge as possible roots
  while(nodes.hasNext())
  {
    nodeid_t n = nodes.next().node;
    // insert all nodes to the root candidate list which are part of this component
    if(!orig.getOutgoingEdges(n).empty())
    {
      roots.insert(n);
    }
  }

  nodes.reset();
  while(nodes.hasNext())
  {
    nodeid_t source = nodes.next().node;

    std::vector<nodeid_t> outEdges = orig.getOutgoingEdges(source);
    for(auto target : outEdges)
    {
      Edge e = {source, target};

      // remove the nodes that have an incoming edge from the root list
      roots.erase(target);

      std::vector<Annotation> edgeAnnos = orig.getEdgeAnnotations(e);
      for(auto a : edgeAnnos)
      {
        edgeAnno.addEdgeAnnotation(e, a);
      }
    }
  }

  uint32_t currentOrder = 0;

  // traverse the graph for each sub-component
  for(const auto& startNode : roots)
  {
    unsigned int lastDistance = 0;

    NStack nodeStack;

    enterNode(currentOrder, startNode, startNode, 0, nodeStack);

    CycleSafeDFS dfs(orig, startNode, 1, uintmax);
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

void PrePostOrderStorage::clear()
{
  node2order.clear();
  order2node.clear();
  edgeAnno.clear();
}

void PrePostOrderStorage::enterNode(uint32_t& currentOrder, nodeid_t nodeID, nodeid_t rootNode,
                                        int level, NStack& nodeStack)
{
  NodeStackEntry newEntry;
  newEntry.id = nodeID;
  newEntry.order.pre = currentOrder++;
  newEntry.order.level = level;

  nodeStack.push(newEntry);
}

void PrePostOrderStorage::exitNode(uint32_t& currentOrder, NStack &nodeStack)
{
  // find the correct pre/post entry and update the post-value
  auto& entry = nodeStack.top();
  entry.order.post = currentOrder++;

  node2order.insert2(entry.id, entry.order);
  order2node[entry.order] = entry.id;

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
  if(edge.source == edge.target)
  {
    return 0;
  }

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

std::vector<nodeid_t> PrePostOrderStorage::getOutgoingEdges(nodeid_t node) const
{
  std::vector<nodeid_t> result;
  result.reserve(10);

  auto connectedIt = findConnected(node, 1, 1);
  for(auto c=connectedIt->next(); c.first; c=connectedIt->next())
  {
    result.push_back(c.second);
  }

  return result;
}

std::vector<nodeid_t> PrePostOrderStorage::getIncomingEdges(nodeid_t node) const
{
  std::set<nodeid_t> sources;
  auto itRange = node2order.equal_range(node);
  for(auto itTarget=itRange.first; itTarget != itRange.second; itTarget++)
  {
    const auto& targetOrder = itTarget->second;
    // get all nodes that are potential predecessors
    PrePost minPrePost = {0, 0, 0};
    PrePost maxPrePost = {targetOrder.pre-1, uintmax, std::numeric_limits<int32_t>::max()};
    auto itMin = order2node.lower_bound(minPrePost);
    auto itMax = order2node.upper_bound(maxPrePost);
    for(auto itSource=itMin; itSource != itMax; itSource++)
    {
      const PrePost& sourceOrder = itSource->first;
      if(sourceOrder.level == targetOrder.level-1
         && targetOrder.post < sourceOrder.post
         && sourceOrder.pre < targetOrder.pre)
      {
        sources.insert(itSource->second);
      }
    }
  }

  std::vector<nodeid_t> result;
  result.reserve(sources.size());
  for(auto n : sources)
  {
    result.push_back(n);
  }
  return result;

}



PrePostIterator::PrePostIterator(const PrePostOrderStorage &storage, std::uint32_t startNode, unsigned int minDistance, unsigned int maxDistance)
  : storage(storage), startNode(startNode),
    minDistance(minDistance), maxDistance(maxDistance)
{
  init();
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
      const auto& currentPre = currentNode->first.pre;
      const auto& currentPost = currentNode->first.post;
      const auto& currentLevel = currentNode->first.level;

      int diffLevel = currentLevel - startLevel;

      // check post order and level as well
      if(currentPost <= maximumPost && minDistance <= diffLevel && diffLevel <= maxDistance
         && visited.find(currentNode->second) == visited.end())
      {
        // success
        result.first = true;
        result.second = currentNode->second;

        visited.insert(result.second);

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

void PrePostIterator::init()
{
  auto subComponentsLower = storage.node2order.lower_bound(startNode);
  auto subComponentsUpper = storage.node2order.upper_bound(startNode);

  for(auto it=subComponentsLower; it != subComponentsUpper; it++)
  {
    auto pre = it->second.pre;
    auto post = it->second.post;
    auto lowerIt = storage.order2node.lower_bound({pre, 0, 0});
    auto upperIt = storage.order2node.upper_bound({post, uintmax, std::numeric_limits<int32_t>::max()});

    ranges.push({lowerIt, upperIt, post, it->second.level});
  }
  if(!ranges.empty())
  {
    currentNode = ranges.top().lower;
  }
}

void PrePostIterator::reset()
{
  while(!ranges.empty())
  {
    ranges.pop();
  }

  visited.clear();

  init();
}

PrePostIterator::~PrePostIterator()
{

}
