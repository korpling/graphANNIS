#ifndef FALLBACKEDGEDB_H
#define FALLBACKEDGEDB_H

#include <stx/btree_map>
#include <stx/btree_multimap>
#include "../edgedb.h"
#include "../db.h"
#include "../comparefunctions.h"

#include <stack>

namespace annis
{


class FallbackEdgeDB : public EdgeDB
{
friend class FallbackReachableIterator;

public:
  FallbackEdgeDB(StringStorage& strings, const Component& component);

  virtual void addEdge(const Edge& edge);
  virtual void addEdgeAnnotation(const Edge &edge, const Annotation& anno);
  virtual void clear();

  virtual std::string getName() {return "fallback";}
  virtual const Component& getComponent();

  virtual bool isConnected(const Edge& edge, unsigned int distance) const;
  virtual AnnotationIterator* findConnected(std::uint32_t sourceNode,
                                           unsigned int minDistance = 1,
                                           unsigned int maxDistance = 1) const;
  virtual std::vector<Annotation> getEdgeAnnotations(const Edge &edge);

  virtual bool load(std::string dirPath);
  virtual bool save(std::string dirPath);

  virtual std::uint32_t numberOfEdges() const;
  virtual std::uint32_t numberOfEdgeAnnotations() const;

private:
  StringStorage& strings;
  Component component;


  stx::btree_map<std::uint32_t, std::uint32_t> edges;
  stx::btree_multimap<Edge, Annotation, compEdges> edgeAnnotations;


};


/** A depth first traverser */
class FallbackReachableIterator : public AnnotationIterator
{
  typedef stx::btree_map<std::uint32_t, std::uint32_t>::const_iterator EdgeIt;

public:

  FallbackReachableIterator(const FallbackEdgeDB& edb, std::uint32_t startNode, unsigned int minDistance, unsigned int maxDistance);

  virtual bool hasNext();
  virtual Match next();
private:

  const FallbackEdgeDB& edb;

  std::stack<EdgeIt> traversalStack;
  unsigned int minDistance;
  unsigned int maxDistance;

  std::uint32_t nodeNameID;
  std::uint32_t nodeNamespaceID;
  std::uint32_t emptyValID;
};

} // end namespace annis
#endif // FALLBACKEDGEDB_H
