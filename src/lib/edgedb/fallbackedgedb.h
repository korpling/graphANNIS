#ifndef FALLBACKEDGEDB_H
#define FALLBACKEDGEDB_H

#include <stx/btree_map>
#include <stx/btree_multimap>
#include "../edgedb.h"


namespace annis
{
class FallbackEdgeDB : public EdgeDB
{
public:
  FallbackEdgeDB(const Component& component);

  virtual void addEdge(Edge edge);
  virtual void addEdgeAnnotation(Edge edge, const Annotation& anno);
  virtual void clear();

  virtual std::string getName() {return "fallback";}
  virtual const Component& getComponent();

  std::vector<Annotation> getEdgeAnnotations(Edge edge);
private:
  Component component;

  stx::btree_map<std::uint32_t, std::uint32_t> edges;
  stx::btree_multimap<Edge, Annotation> edgeAnnotations;


};
} // end namespace annis
#endif // FALLBACKEDGEDB_H
