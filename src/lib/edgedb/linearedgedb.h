#ifndef LINEAREDGEDB_H
#define LINEAREDGEDB_H

#include <stx/btree_map>
#include "../edgedb.h"
#include "../edgeannotationstorage.h"

namespace annis
{


class LinearEdgeDB : public ReadableGraphStorage
{
friend class LinearIterator;

public:
  LinearEdgeDB(StringStorage& strings, const Component& component);

  virtual void clear();
  virtual void copy(const DB& db, const ReadableGraphStorage& orig);

  virtual bool isConnected(const Edge& edge, unsigned int minDistance, unsigned int maxDistance) const;
  virtual std::unique_ptr<EdgeIterator> findConnected(
                                           nodeid_t sourceNode,
                                           unsigned int minDistance,
                                           unsigned int maxDistance) const;

  virtual int distance(const Edge &edge) const;

  virtual bool load(std::string dirPath);
  virtual bool save(std::string dirPath);

  virtual std::vector<Annotation> getEdgeAnnotations(const Edge& edge) const
  {
    return edgeAnno.getEdgeAnnotations(edge);
  }
  virtual std::vector<nodeid_t> getOutgoingEdges(nodeid_t node) const;
  virtual std::vector<nodeid_t> getIncomingEdges(nodeid_t node) const;

  virtual std::uint32_t numberOfEdges() const
  {
    return node2pos.size();
  }
  virtual std::uint32_t numberOfEdgeAnnotations() const
  {
    return edgeAnno.numberOfEdgeAnnotations();
  }

  virtual ~LinearEdgeDB();

private:
  stx::btree_map<nodeid_t, RelativePosition> node2pos;
  std::map<nodeid_t, std::vector<nodeid_t> > nodeChains;

  EdgeAnnotationStorage edgeAnno;
};

class LinearIterator : public EdgeIterator
{
public:

  LinearIterator(const LinearEdgeDB& edb, nodeid_t startNode, unsigned int minDistance, unsigned int maxDistance);

  virtual std::pair<bool, nodeid_t> next();

  virtual void reset();

  virtual ~LinearIterator();
private:

  const LinearEdgeDB& edb;
  unsigned int minDistance;
  unsigned int maxDistance;
  nodeid_t startNode;

  const std::vector<nodeid_t>* chain;
  uint32_t currentPos;
  uint32_t endPos;

};

} // end namespace annis

#endif // LINEAREDGEDB_H
