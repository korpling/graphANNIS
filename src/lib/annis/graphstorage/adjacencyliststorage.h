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

#include <cereal/types/polymorphic.hpp>       // for CEREAL_REGISTER_TYPE


#include <annis/types.h>                      // for Edge, nodeid_t, operator<
#include <annis/serializers.h>


#include <annis/annostorage.h>                // for AnnoStorage
#include <annis/graphstorage/graphstorage.h>  // for WriteableGraphStorage
#include <google/btree_container.h>           // for btree_unique_container<...
#include <google/btree_set.h>                 // for btree_set
#include <stddef.h>                           // for size_t
#include <annis/annosearch/estimatedsearch.h>


#include <memory>                             // for unique_ptr
#include <vector>                             // for vector
namespace annis { class DB; }
namespace annis { class EdgeIterator; }
namespace annis { class StringStorage; }


namespace annis
{

class AdjacencyListStorage : public WriteableGraphStorage
{

public:

  template<typename Key> using set_t = btree::btree_set<Key>;

  class NodeIt : public EstimatedSearch
  {
  public:
    NodeIt(set_t<Edge>::const_iterator itStart, set_t<Edge>::const_iterator itEnd);
    virtual bool next(Match& m) override;
    virtual void reset() override;

    virtual ~NodeIt() {}
  private:
    set_t<Edge>::const_iterator it;
    set_t<Edge>::const_iterator itStart;
    set_t<Edge>::const_iterator itEnd;

    boost::optional<nodeid_t> lastNode;
  };

  AdjacencyListStorage() {}

  virtual void copy(const DB& db, const ReadableGraphStorage& orig) override;

  virtual void addEdge(const Edge& edge) override;
  virtual void addEdgeAnnotation(const Edge &edge, const Annotation& anno) override;

  virtual void deleteEdge(const Edge& edge) override;
  virtual void deleteNode(nodeid_t node) override;
  virtual void deleteEdgeAnnotation(const Edge& edge, const AnnotationKey& anno) override;

  virtual void clear() override;

  virtual bool isConnected(const Edge& edge, unsigned int minDistance, unsigned int maxDistance) const override;
  virtual std::unique_ptr<EdgeIterator> findConnected(nodeid_t sourceNode,
                                           unsigned int minDistance = 1,
                                           unsigned int maxDistance = 1) const override;

  virtual int distance(const Edge &edge) const override;

  virtual std::vector<Annotation> getEdgeAnnotations(const Edge &edge) const override;
  virtual std::vector<nodeid_t> getOutgoingEdges(nodeid_t node) const override;

  set_t<Edge>::const_iterator getEdgesBegin() const
  {
    return edges.begin();
  }
  set_t<Edge>::const_iterator getEdgesEnd() const
  {
    return edges.end();
  }


  virtual size_t numberOfEdges() const override;
  virtual size_t numberOfEdgeAnnotations() const override;

  virtual const BTreeMultiAnnoStorage<Edge>& getAnnoStorage() const override
  {
    return edgeAnnos;
  }

  virtual std::shared_ptr<AnnoIt> getSourceNodeIterator() const override
  {
    return std::make_shared<NodeIt>(getEdgesBegin(), getEdgesEnd());
  }

  virtual void calculateStatistics(const StringStorage& strings) override;

  virtual size_t estimateMemorySize() override;

  template<class Archive>
  void serialize(Archive & archive)
  {
    archive(cereal::base_class<WriteableGraphStorage>(this),
            edges, inverseEdges, edgeAnnos);
  }

private:

  set_t<Edge> edges;
  set_t<Edge> inverseEdges;

  BTreeMultiAnnoStorage<Edge> edgeAnnos;

private:
  friend class cereal::access;

};


} // end namespace annis


#include <cereal/archives/binary.hpp>
#include <cereal/archives/xml.hpp>
#include <cereal/archives/json.hpp>

CEREAL_REGISTER_TYPE(annis::AdjacencyListStorage)
