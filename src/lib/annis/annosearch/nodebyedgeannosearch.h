#pragma once

#include <annis/annosearch/annotationsearch.h>
#include <annis/graphstorage/graphstorage.h>

namespace annis
{
class NodeByEdgeAnnoSearch : public AnnoIt
{

  using ItType = BTreeMultiAnnoStorage<Edge>::InverseAnnoMap_t::const_iterator;

public:
  NodeByEdgeAnnoSearch(const ReadableGraphStorage& gs);

  virtual bool next(Match& m) override;
  virtual void reset() override;

  virtual ~NodeByEdgeAnnoSearch();
private:
  const ReadableGraphStorage& gs;
};

}

