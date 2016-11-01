#pragma once

#include <annis/graphstorage/graphstorage.h>
#include <annis/edgeannotationstorage.h>
#include <annis/util/dfs.h>
#include <annis/annosearch/exactannovaluesearch.h>
#include <annis/annosearch/exactannokeysearch.h>

#include <set>
#include <google/btree_map.h>
#include <google/btree_set.h>
#include <stack>
#include <list>


#include <fstream>

#include <cereal/types/polymorphic.hpp>

#include <annis/serializers_cereal.h>
#include <annis/util/size_estimator.h>

namespace annis
{

template<typename order_t, typename level_t>
struct PrePost
{
  order_t pre;
  order_t post;
  level_t level;
};

template<class Archive, typename order_t, typename level_t>
void serialize(Archive & archive,
               PrePost<order_t, level_t> & m)
{
  archive(m.pre, m.post, m.level);
}

} // end namespace annis

namespace std
{
template<typename order_t, typename level_t>
struct less<annis::PrePost<order_t, level_t> >
{
  bool operator()(const struct annis::PrePost<order_t, level_t> &a, const struct annis::PrePost<order_t, level_t> &b) const
  {
    // compare by pre-order
    if(a.pre < b.pre) {return true;} else if(a.pre > b.pre) {return false;}

    // compare by post-order
    if(a.post < b.post) {return true;} else if(a.post > b.post) {return false;}

    // compare by level
    if(a.level < b.level) {return true;} else if(a.level > b.level) {return false;}

    // they are equal
    return false;
  }
};

} // end namespace std

namespace annis
{

template<typename order_t, typename level_t>
struct SearchRange
{
  typename btree::btree_map<PrePost<order_t, level_t>, nodeid_t>::const_iterator lower;
  typename btree::btree_map<PrePost<order_t, level_t>, nodeid_t>::const_iterator upper;
  order_t maximumPost;
  level_t startLevel;
};

template<typename order_t, typename level_t>
struct NodeStackEntry
{
  nodeid_t id;
  PrePost<order_t, level_t> order;
};

template<typename order_t, typename level_t>
class PrePostOrderStorage : public ReadableGraphStorage
{

public:

  template<typename Key, typename Value>
  using map_t = btree::btree_map<Key, Value>;

  template<typename Key, typename Value>
  using multimap_t = btree::btree_multimap<Key, Value>;

  class PrePostIterator : public EdgeIterator
  {

    using PrePostOrderStorageSpec = PrePostOrderStorage<order_t, level_t>;
    using SearchRangeSpec = SearchRange<order_t, level_t>;
    using OrderIt = typename map_t<PrePost<order_t, level_t>, nodeid_t>::const_iterator;
  public:

    PrePostIterator(const PrePostOrderStorageSpec& storage,
                    const nodeid_t& startNode,
                    const unsigned int& minDistance,
                    const unsigned int& maxDistance)
      : storage(storage), startNode(startNode),
        minDistance(minDistance), maxDistance(maxDistance)
    {
      init();
    }

    virtual std::pair<bool, nodeid_t> next() override
    {
      std::pair<bool, nodeid_t> result(0, false);

      while(!ranges.empty())
      {
        const auto& upper = ranges.top().upper;
        const auto& maximumPost = ranges.top().maximumPost;
        const auto& startLevel = ranges.top().startLevel;

        while(currentNode != upper && currentNode->first.pre <= maximumPost)
        {
          const auto& currentPre = currentNode.key().pre;
          const auto& currentPost = currentNode.key().post;
          const auto& currentLevel = currentNode.key().level;

          unsigned int diffLevel = std::abs(currentLevel - startLevel);

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

    virtual void reset() override
    {
      while(!ranges.empty())
      {
        ranges.pop();
      }

      visited.clear();

      init();
    }

    virtual ~PrePostIterator()
    {

    }

  private:

    const PrePostOrderStorageSpec& storage;
    const nodeid_t startNode;
    const unsigned int minDistance;
    const unsigned int maxDistance;


    std::stack<SearchRangeSpec, std::list<SearchRangeSpec> > ranges;
    OrderIt currentNode;

    btree::btree_set<nodeid_t> visited;

  private:
    void init()
    {
      auto subComponentBegin = storage.node2order.lower_bound(startNode);

      for(auto it=subComponentBegin; it != storage.node2order.end() && it->first == startNode; it++)
      {
        const auto& pre = it->second.pre;
        const auto& post = it->second.post;

        ranges.push({storage.order2node.lower_bound({pre, 0, 0}),
                     storage.order2node.end(),
                     post, it->second.level});
      }
      if(!ranges.empty())
      {
        currentNode = ranges.top().lower;
      }
    }

  };


using NStack = std::stack<NodeStackEntry<order_t, level_t>, std::list<NodeStackEntry<order_t, level_t> > >;
using PrePostSpec = PrePost<order_t, level_t>;

public:
  PrePostOrderStorage()
  {

  }

  virtual ~PrePostOrderStorage()
  {

  }

  template<class Archive>
  void serialize(Archive & archive)
  {
    archive(cereal::base_class<ReadableGraphStorage>(this),
            edgeAnno, node2order, order2node);
  }

  virtual void copy(const DB& db, const ReadableGraphStorage& orig) override
  {
    clear();

    // find all roots of the component
    std::set<nodeid_t> roots;
    ExactAnnoKeySearch nodes(db, annis_ns, annis_node_name);
    Match match;
    // first add all nodes that are a source of an edge as possible roots
    while(nodes.next(match))
    {
      nodeid_t n = match.node;
      // insert all nodes to the root candidate list which are part of this component
      if(!orig.getOutgoingEdges(n).empty())
      {
        roots.insert(n);
      }
    }

    nodes.reset();
    while(nodes.next(match))
    {
      nodeid_t source = match.node;

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

    order_t currentOrder = 0;

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

    stat = orig.getStatistics();
  }

  virtual void clear() override
  {
    node2order.clear();
    order2node.clear();
    edgeAnno.clear();
  }

  virtual std::vector<Annotation> getEdgeAnnotations(const Edge& edge) const override
  {
    return edgeAnno.getEdgeAnnotations(edge);
  }

  virtual bool isConnected(const Edge& edge, unsigned int minDistance = 1, unsigned int maxDistance = 1) const override
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
          unsigned int diffLevel = std::abs(itTarget->second.level - itSource->second.level);
          if(minDistance <= diffLevel && diffLevel <= maxDistance)
          {
            return true;
          }
        }
      }
    }
    return false;
  }
  virtual int distance(const Edge &edge) const override
  {
    if(edge.source == edge.target)
    {
      return 0;
    }

    const auto itSourceBegin = node2order.lower_bound(edge.source);
    const auto itSourceEnd = node2order.upper_bound(edge.source);

    bool wasFound = false;
    level_t minLevel = std::numeric_limits<level_t>::max();

    for(auto itSource=itSourceBegin; itSource != itSourceEnd; itSource++)
    {
      auto itTargetRange = node2order.equal_range(edge.target);
      for(auto itTarget=itTargetRange.first; itTarget != itTargetRange.second; itTarget++)
      {
        if(itSource->second.pre <= itTarget->second.pre
           && itTarget->second.post <= itSource->second.post)
        {
          // check the level
          level_t diffLevel = (itTarget->second.level - itSource->second.level);
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
  virtual std::unique_ptr<EdgeIterator> findConnected(nodeid_t sourceNode,
                                           unsigned int minDistance = 1,
                                           unsigned int maxDistance = 1) const override
  {
    return std::unique_ptr<EdgeIterator>(
          new PrePostIterator(*this, sourceNode, minDistance, maxDistance));
  }

  virtual std::vector<nodeid_t> getOutgoingEdges(nodeid_t node) const override
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

  virtual size_t numberOfEdges() const override
  {
    return order2node.size();
  }
  virtual size_t numberOfEdgeAnnotations() const override
  {
    return edgeAnno.numberOfEdgeAnnotations();
  }

  virtual size_t estimateMemorySize() override
  {
    return
        size_estimation::element_size(node2order)
        + size_estimation::element_size(order2node)
        + edgeAnno.estimateMemorySize()
        + sizeof(PrePostOrderStorage<order_t, level_t>);
  }

private:
  multimap_t<nodeid_t, PrePostSpec> node2order;
  map_t<PrePostSpec, nodeid_t> order2node;
  EdgeAnnotationStorage edgeAnno;

  void enterNode(order_t& currentOrder, nodeid_t nodeID, nodeid_t /* rootNode */, level_t level, NStack &nodeStack)
  {
    NodeStackEntry<order_t, level_t> newEntry;
    newEntry.id = nodeID;
    newEntry.order.pre = currentOrder++;
    newEntry.order.level = level;

    nodeStack.push(newEntry);
  }

  void exitNode(order_t &currentOrder, NStack &nodeStack)
  {
    // find the correct pre/post entry and update the post-value
    auto& entry = nodeStack.top();
    entry.order.post = currentOrder++;

    node2order.insert({entry.id, entry.order});
    order2node[entry.order] = entry.id;

    nodeStack.pop();
  }

};

} // end namespace annis


#include <cereal/archives/binary.hpp>
#include <cereal/archives/xml.hpp>
#include <cereal/archives/json.hpp>


CEREAL_REGISTER_TYPE(annis::PrePostOrderStorage<uint32_t, int32_t>)
CEREAL_REGISTER_TYPE(annis::PrePostOrderStorage<uint32_t, int8_t>)
CEREAL_REGISTER_TYPE(annis::PrePostOrderStorage<uint16_t, int32_t>)
CEREAL_REGISTER_TYPE(annis::PrePostOrderStorage<uint16_t, int8_t>)

