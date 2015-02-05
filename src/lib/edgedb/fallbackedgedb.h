#ifndef FALLBACKEDGEDB_H
#define FALLBACKEDGEDB_H

#include <stx/btree_map>
#include <stx/btree_multimap>
#include <stx/btree_set>
#include "../graphstorage.h"
#include "../db.h"
#include "../comparefunctions.h"
#include "../edgeannotationstorage.h"

#include <stack>
#include <list>
#include <set>
#include <map>

namespace annis
{

class FallbackEdgeDB : public WriteableGraphStorage
{

public:
  FallbackEdgeDB(StringStorage& strings, const Component& component);

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

  stx::btree_set<Edge>::const_iterator getEdgesBegin()
  {
    return edges.begin();
  }
  stx::btree_set<Edge>::const_iterator getEdgesEnd()
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

  stx::btree_set<Edge> edges;
  EdgeAnnotationStorage edgeAnnos;

};


} // end namespace annis
#endif // FALLBACKEDGEDB_H
