#pragma once

#include <annis/annosearch/annotationsearch.h>
#include <annis/graphstorage/graphstorage.h>
#include <functional>

namespace annis
{
class NodeByEdgeAnnoSearch : public AnnoIt
{

  using ItType = BTreeMultiAnnoStorage<Edge>::InverseAnnoMap_t::const_iterator;
  using Range = std::pair<ItType, ItType>;

public:
  NodeByEdgeAnnoSearch(const ReadableGraphStorage& gs, std::set<Annotation> validEdgeAnnos,
                       std::function<std::list<Match> (nodeid_t)> nodeAnnoMatchGenerator);

  virtual bool next(Match& m) override;
  virtual void reset() override;

  virtual ~NodeByEdgeAnnoSearch();
private:
  std::function<std::list<Match> (nodeid_t)> nodeAnnoMatchGenerator;

  std::list<Match> currentMatchBuffer;

  std::list<Range> searchRanges;
  std::list<Range>::const_iterator currentRange;
  ItType it;

private:
  bool nextMatchBuffer();
};

}

