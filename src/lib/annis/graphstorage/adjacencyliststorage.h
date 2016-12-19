#pragma once

#include <annis/graphstorage/graphstorage.h>
#include <annis/db.h>
#include <annis/util/comparefunctions.h>
#include <annis/annostorage.h>

#include <stack>
#include <list>
#include <set>
#include <map>
#include <sstream>

#include <google/btree_set.h>

#include <cereal/types/polymorphic.hpp>
#include <annis/serializers.h>

namespace annis
{

class AdjacencyListStorage : public WriteableGraphStorage
{

public:

  template<typename Key> using set_t = btree::btree_set<Key>;


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

  set_t<Edge>::const_iterator getEdgesBegin()
  {
    return edges.begin();
  }
  set_t<Edge>::const_iterator getEdgesEnd()
  {
    return edges.end();
  }


  virtual size_t numberOfEdges() const override;
  virtual size_t numberOfEdgeAnnotations() const override;

  virtual size_t guessMaxAnnoCount(const StringStorage& strings, const Annotation& anno) const override
  {
    return edgeAnnos.guessMaxCount(strings, anno);
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
