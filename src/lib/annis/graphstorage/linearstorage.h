#pragma once

#include <annis/graphstorage/graphstorage.h>
#include <annis/edgeannotationstorage.h>
#include <annis/util/dfs.h>
#include <annis/annosearch/exactannokeysearch.h>

#include <fstream>
#include <set>
#include <limits>

#include <cereal/types/vector.hpp>
#include <annis/serializers_cereal.h>

#include <annis/util/size_estimator.h>

#include <boost/format.hpp>

#include <google/btree_map.h>


namespace annis
{


template<typename pos_t>
class LinearStorage : public ReadableGraphStorage
{
public:

  template<typename Key, typename Value>
  using map_t = btree::btree_map<Key, Value>;

  class LinearIterator : public EdgeIterator
  {
  public:


    LinearIterator(const LinearStorage& gs, nodeid_t startNode, unsigned int minDistance, unsigned int maxDistance)
      : gs(gs), minDistance(minDistance), maxDistance(maxDistance), startNode(startNode),
        chain(nullptr)
    {
      reset();
    }

    virtual std::pair<bool, nodeid_t> next() override
    {
      bool found = false;
      nodeid_t node = 0;
      if(chain != nullptr && currentPos <= endPos && currentPos < chain->size())
      {
        found = true;
        node = chain->at(currentPos);
        chain->at(currentPos);
        currentPos++;
      }
      return std::pair<bool, nodeid_t>(found, node);
    }

    virtual void reset() override
    {
      typedef typename map_t<nodeid_t, RelativePosition<pos_t> >::const_iterator PosIt;
      typedef map_t<nodeid_t, std::vector<nodeid_t> >::const_iterator NodeChainIt;

      PosIt posSourceIt = gs.node2pos.find(startNode);
      if(posSourceIt != gs.node2pos.end())
      {
        const RelativePosition<pos_t>& relPos = posSourceIt->second;
        currentPos = relPos.pos;
        NodeChainIt itNodeChain = gs.nodeChains.find(relPos.root);
        if(itNodeChain != gs.nodeChains.end())
        {
          chain = &(itNodeChain->second);
        }

        // define where to stop
        if(maxDistance == uintmax)
        {
          endPos = std::numeric_limits<pos_t>::max();
        }
        else
        {
          endPos = currentPos + maxDistance;
        }
        // add the minium distance
        currentPos = currentPos + minDistance;

      }
    }

    virtual ~LinearIterator()
    {

    }

  private:

    const LinearStorage& gs;
    unsigned int minDistance;
    unsigned int maxDistance;
    nodeid_t startNode;

    const std::vector<nodeid_t>* chain;
    unsigned int currentPos;
    unsigned int endPos;

  };


public:
  LinearStorage(const Component& component)
    : component(component)
  {

  }

  virtual void clear() override
  {
    edgeAnno.clear();
    node2pos.clear();
    nodeChains.clear();
  }

  virtual void copy(const DB& db, const ReadableGraphStorage& orig) override
  {
    // find all root nodes
    std::set<nodeid_t> roots;

    // add all nodes to root list
    ExactAnnoKeySearch nodes(db, annis_ns, annis_node_name);

    Match match;
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


    for(auto& rootNode : roots)
    {
      // iterate over all edges beginning from the root
      nodeChains[rootNode] = std::vector<nodeid_t>();
      std::vector<nodeid_t>& chain = nodeChains[rootNode];
      chain.push_back(rootNode);
      node2pos[rootNode] = {rootNode, (pos_t) (chain.size()-1)};

      CycleSafeDFS it(orig, rootNode, 1, uintmax);

      uint32_t pos=1;
      for(std::pair<bool, nodeid_t> node = it.next(); node.first; node = it.next(), pos++)
      {
        chain.push_back(node.second);
        node2pos[node.second] = {rootNode, (pos_t)(chain.size()-1)};
      }
    }

    stat = orig.getStatistics();
  }

  virtual bool isConnected(const Edge& edge, unsigned int minDistance, unsigned int maxDistance) const override
  {
    typedef typename map_t<nodeid_t, RelativePosition<pos_t> >::const_iterator PosIt;

    PosIt posSourceIt = node2pos.find(edge.source);
    PosIt posTargetIt = node2pos.find(edge.target);
    if(posSourceIt != node2pos.end() && posTargetIt != node2pos.end())
    {
      auto& posSource = posSourceIt->second;
      auto& posTarget = posTargetIt->second;
      if(posSource.root == posTarget.root && posSource.pos <= posTarget.pos)
      {
        unsigned int diff = posTarget.pos > posSource.pos ?
              posTarget.pos - posSource.pos
            : posSource.pos - posTarget.pos;

        if(diff >= minDistance && diff <= maxDistance)
        {
          return true;
        }
      }
    }
    return false;
  }
  virtual std::unique_ptr<EdgeIterator> findConnected(
                                           nodeid_t sourceNode,
                                           unsigned int minDistance,
                                           unsigned int maxDistance) const override
  {
    return std::unique_ptr<EdgeIterator>(new LinearIterator(*this, sourceNode, minDistance, maxDistance));
  }

  virtual int distance(const Edge &edge) const override
  {
    typedef typename map_t<nodeid_t, RelativePosition<pos_t> >::const_iterator PosIt;

    PosIt posSourceIt = node2pos.find(edge.source);
    PosIt posTargetIt = node2pos.find(edge.target);
    if(posSourceIt != node2pos.end() && posTargetIt != node2pos.end())
    {
      auto& posSource = posSourceIt->second;
      auto& posTarget = posTargetIt->second;
      if(posSource.root == posTarget.root && posSource.pos <= posTarget.pos)
      {
        int diff = posTarget.pos - posSource.pos;
        if(diff >= 0)
        {
          return diff;
        }
      }
    }
    return -1;
  }

  template<class Archive>
  void serialize(Archive & archive)
  {
    ReadableGraphStorage::serialize(archive);
    archive(edgeAnno, node2pos, nodeChains);
  }


  virtual std::vector<Annotation> getEdgeAnnotations(const Edge& edge) const override
  {
    return edgeAnno.getEdgeAnnotations(edge);
  }
  virtual std::vector<nodeid_t> getOutgoingEdges(nodeid_t node) const override
  {
    std::vector<nodeid_t> result;
    auto it = node2pos.find(node);
    if(it != node2pos.end())
    {
      auto pos = it->second;
      auto chainIt = nodeChains.find(pos.root);
      if(chainIt != nodeChains.end())
      {
        const std::vector<nodeid_t>& chain = chainIt->second;
        if(pos.pos < (chain.size()-1))
        {
          result.push_back(chain[pos.pos+1]);
        }
      }
    }
    return result;
  }

  virtual size_t numberOfEdges() const override
  {
    return node2pos.size();
  }
  virtual size_t numberOfEdgeAnnotations() const override
  {
    return edgeAnno.numberOfEdgeAnnotations();
  }



  virtual size_t estimateMemorySize() override
  {
    return
        size_estimation::element_size(node2pos)
        + size_estimation::element_size(nodeChains)
        + edgeAnno.estimateMemorySize()
        + sizeof(LinearStorage<pos_t>);
  }

  virtual ~LinearStorage()
  {

  }

private:
  const Component& component;
  map_t<nodeid_t, RelativePosition<pos_t>> node2pos;
  map_t<nodeid_t, std::vector<nodeid_t> > nodeChains;

  EdgeAnnotationStorage edgeAnno;
};

} // end namespace annis

