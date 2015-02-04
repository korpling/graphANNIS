#ifndef COVERAGEEDB_H
#define COVERAGEEDB_H

#include "fallbackedgedb.h"

#include <unordered_map>

namespace annis
{

class CoverageEdgeDB  : public FallbackEdgeDB
{
public:
  CoverageEdgeDB(StringStorage& strings, const Component& component);

  virtual void calculateIndex();

  virtual bool save(std::string dirPath);
  virtual bool load(std::string dirPath);

  virtual std::vector<nodeid_t> getIncomingEdges(nodeid_t node) const;

  virtual int distance(const Edge &edge) const;
  virtual bool isConnected(const Edge& edge, unsigned int minDistance, unsigned int maxDistance) const;

  virtual ~CoverageEdgeDB();

private:

  std::unordered_multimap<nodeid_t, nodeid_t> coveringNodes;

};

} // end namespace annis

#endif // COVERAGEEDB_H
