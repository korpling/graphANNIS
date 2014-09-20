#ifndef LINEAREDGEDB_H
#define LINEAREDGEDB_H

#include <stx/btree_map>
#include "../edgedb.h"
#include "fallbackedgedb.h"

namespace annis
{


class LinearEdgeDB : public FallbackEdgeDB
{
friend class LinearIterator;

public:
  LinearEdgeDB(StringStorage& strings, const Component& component);
  virtual void clear();
  virtual void calculateIndex();

  virtual bool isConnected(const Edge& edge, unsigned int minDistance, unsigned int maxDistance) const;
  virtual EdgeIterator* findConnected(
                                           nodeid_t sourceNode,
                                           unsigned int minDistance,
                                           unsigned int maxDistance) const;

  virtual bool load(std::string dirPath);
  virtual bool save(std::string dirPath);

  virtual ~LinearEdgeDB();

private:
  stx::btree_map<nodeid_t, RelativePosition> node2pos;
  stx::btree_map<RelativePosition, nodeid_t, compRelativePosition> pos2node;
};

class LinearIterator : public EdgeIterator
{
public:

  LinearIterator(const LinearEdgeDB& edb, std::uint32_t startNode, unsigned int minDistance, unsigned int maxDistance);

  virtual std::pair<bool, nodeid_t> next();
private:

  const LinearEdgeDB& edb;

  RelativePosition currentPos;
  u_int32_t endPos;

};

} // end namespace annis

#endif // LINEAREDGEDB_H
