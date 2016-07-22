#pragma once

#include <stx/btree_map>
#include <stx/btree_multimap>
#include <stx/btree_set>
#include <annis/graphstorage/graphstorage.h>
#include <annis/db.h>
#include <annis/util/comparefunctions.h>
#include <annis/edgeannotationstorage.h>

#include <stack>
#include <list>
#include <set>
#include <map>

#include <google/btree_set.h>

namespace annis
{

class AdjacencyListStorage : public WriteableGraphStorage
{

public:

  template<typename Key> using set_t = btree::btree_set<Key>;

  AdjacencyListStorage(StringStorage& strings, const Component& component);

  virtual void copy(const DB& db, const ReadableGraphStorage& orig);

  virtual void addEdge(const Edge& edge);
  virtual void addEdgeAnnotation(const Edge &edge, const Annotation& anno);
  virtual void clear();

  virtual bool isConnected(const Edge& edge, unsigned int minDistance, unsigned int maxDistance) const;
  virtual std::unique_ptr<EdgeIterator> findConnected(nodeid_t sourceNode,
                                           unsigned int minDistance = 1,
                                           unsigned int maxDistance = 1) const;

  virtual int distance(const Edge &edge) const;

  virtual std::vector<Annotation> getEdgeAnnotations(const Edge &edge) const;
  virtual std::vector<nodeid_t> getOutgoingEdges(nodeid_t node) const;

  set_t<Edge>::const_iterator getEdgesBegin()
  {
    return edges.begin();
  }
  set_t<Edge>::const_iterator getEdgesEnd()
  {
    return edges.end();
  }

  virtual bool load(std::string dirPath);
  virtual bool save(std::string dirPath);

  virtual std::uint32_t numberOfEdges() const;
  virtual std::uint32_t numberOfEdgeAnnotations() const;
  const Component& getComponent() { return component;}

  virtual void calculateStatistics();

private:
  StringStorage& strings;
  Component component;

  set_t<Edge> edges;
  EdgeAnnotationStorage edgeAnnos;

};


} // end namespace annis
