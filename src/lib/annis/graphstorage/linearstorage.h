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

#include <annis/annosearch/exactannokeysearch.h>  // for ExactAnnoKeySearch
#include <annis/annostorage.h>                    // for AnnoStorage
#include <annis/graphstorage/graphstorage.h>      // for ReadableGraphStorage
#include <annis/util/dfs.h>                       // for CycleSafeDFS
#include <annis/util/size_estimator.h>            // for element_size
#include <google/btree.h>                         // for btree_iterator
#include <google/btree_container.h>               // for btree_unique_contai...
#include <google/btree_map.h>                     // for btree_map
#include <stddef.h>                               // for size_t
#include <stdint.h>                               // for uint32_t, uint16_t
#include <cereal/types/base_class.hpp>            // for base_class
#include <cereal/types/polymorphic.hpp>           // for CEREAL_REGISTER_TYPE
#include <limits>                                 // for numeric_limits
#include <memory>                                 // for unique_ptr
#include <set>                                    // for set, allocator
#include <utility>                                // for pair
#include <vector>                                 // for vector
#include "annis/db.h"                             // for DB
#include "annis/iterators.h"                      // for EdgeIterator
#include "annis/types.h"                          // for nodeid_t, Edge, Rel...
#include <annis/annosearch/estimatedsearch.h>


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

    virtual boost::optional<nodeid_t> next() override
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
      return found ? node : boost::optional<nodeid_t>();
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

  class NodeIt : public BufferedEstimatedSearch
  {
  public:
    using OrderIt = typename map_t<nodeid_t, RelativePosition<pos_t>>::const_iterator;

    NodeIt(std::function<std::list<Annotation> (nodeid_t)> nodeAnnoMatchGenerator, bool maximalOneNodeAnno,
           bool returnsNothing,
           const LinearStorage<pos_t>& storage)
      : BufferedEstimatedSearch(maximalOneNodeAnno, returnsNothing),
        nodeAnnoMatchGenerator(nodeAnnoMatchGenerator),
        it(storage.node2pos.begin()), itStart(storage.node2pos.begin()), itEnd(storage.node2pos.end()),
        maxCount(storage.stat.nodes)
    {

    }


    bool nextMatchBuffer(std::list<Match>& currentMatchBuffer) override
    {
      currentMatchBuffer.clear();
      while(it != itEnd)
      {
        if(!lastNode || *lastNode != it->first)
        {
          if(getConstAnnoValue())
          {
            currentMatchBuffer.push_back({it->first, *getConstAnnoValue()});
          }
          else
          {
            for(const Annotation& anno : nodeAnnoMatchGenerator(it->first))
            {
              currentMatchBuffer.push_back({it->first, anno});
            }
          }

          lastNode = it->first;
          return true;
        }

        it++;
      }
      return false;
    }

    virtual void reset() override
    {
      BufferedEstimatedSearch::reset();
      it = itStart;
      lastNode.reset();
    }

    virtual std::function<std::list<Annotation> (nodeid_t)> getNodeAnnoMatchGenerator() override
    {
      return nodeAnnoMatchGenerator;
    }

    virtual std::int64_t guessMaxCount() const override
    {
      return maxCount;
    }

    virtual ~NodeIt() {}
  private:
    const std::function<std::list<Annotation> (nodeid_t)> nodeAnnoMatchGenerator;

    OrderIt it;
    OrderIt itStart;
    OrderIt itEnd;

    boost::optional<nodeid_t> lastNode;

    std::int64_t maxCount;
  };


public:

  LinearStorage() {}

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
          edgeAnno.addAnnotation(e, a);
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
      for(boost::optional<nodeid_t> node = it.next(); node; node = it.next(), pos++)
      {
        chain.push_back(*node);
        node2pos[*node] = {rootNode, (pos_t)(chain.size()-1)};
      }
    }

    stat = orig.getStatistics();
    calculateStatistics(db.strings);
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
    archive(cereal::base_class<ReadableGraphStorage>(this),
            edgeAnno, node2pos, nodeChains);
  }


  virtual std::vector<Annotation> getEdgeAnnotations(const Edge& edge) const override
  {
    return edgeAnno.getAnnotations(edge);
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
    return edgeAnno.numberOfAnnotations();
  }

  virtual const BTreeMultiAnnoStorage<Edge>& getAnnoStorage() const override
  {
    return edgeAnno;
  }

  virtual std::shared_ptr<EstimatedSearch> getSourceNodeIterator(
      std::function<std::list<Annotation> (nodeid_t)> nodeAnnoMatchGenerator,
      bool maximalOneNodeAnno, bool returnsNothing) const override
  {
    return std::make_shared<NodeIt>(nodeAnnoMatchGenerator, maximalOneNodeAnno, returnsNothing, *this);
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

  map_t<nodeid_t, RelativePosition<pos_t>> node2pos;
  map_t<nodeid_t, std::vector<nodeid_t> > nodeChains;

  BTreeMultiAnnoStorage<Edge> edgeAnno;
};

} // end namespace annis


#include <cereal/archives/binary.hpp>
#include <cereal/archives/xml.hpp>
#include <cereal/archives/json.hpp>

CEREAL_REGISTER_TYPE(annis::LinearStorage<uint32_t>)
CEREAL_REGISTER_TYPE(annis::LinearStorage<uint16_t>)
CEREAL_REGISTER_TYPE(annis::LinearStorage<uint8_t>)
