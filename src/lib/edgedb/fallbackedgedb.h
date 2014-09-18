#ifndef FALLBACKEDGEDB_H
#define FALLBACKEDGEDB_H

#include <stx/btree_map>
#include <stx/btree_multimap>
#include "../edgedb.h"
#include "../comparefunctions.h"


namespace annis
{
class FallbackEdgeDB : public EdgeDB
{
public:
  FallbackEdgeDB(const Component& component);

  virtual void addEdge(const Edge& edge);
  virtual void addEdgeAnnotation(const Edge &edge, const Annotation& anno);
  virtual void clear();

  virtual std::string getName() {return "fallback";}
  virtual const Component& getComponent();

  virtual bool isConnected(const Edge& edge, unsigned int distance) const;
  virtual std::vector<Annotation> getEdgeAnnotations(const Edge &edge);

  virtual bool load(std::string dirPath);
  virtual bool save(std::string dirPath);

  virtual std::uint32_t numberOfEdges() const;
  virtual std::uint32_t numberOfEdgeAnnotations() const;

private:
  Component component;

  stx::btree_map<std::uint32_t, std::uint32_t> edges;
  stx::btree_multimap<Edge, Annotation, compEdges> edgeAnnotations;


};
} // end namespace annis
#endif // FALLBACKEDGEDB_H
