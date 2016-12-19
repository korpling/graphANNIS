#pragma once

#include <annis/annosearch/annotationsearch.h>
#include <annis/graphstorage/graphstorage.h>
#include <functional>

namespace annis
{
class NodeByEdgeAnnoSearch : public AnnoIt
{

  using ItType = BTreeMultiAnnoStorage<Edge>::InverseAnnoMap_t::const_iterator;

public:
  NodeByEdgeAnnoSearch(const ReadableGraphStorage& gs, std::set<Annotation> validEdgeAnnos,
                       std::function<std::list<Match> (nodeid_t)> nodeAnnoMatchGenerator);

  virtual bool next(Match& m) override;
  virtual void reset() override;

  virtual ~NodeByEdgeAnnoSearch();
private:
  const ReadableGraphStorage& gs;
  const std::set<Annotation> validEdgeAnnos;
  std::function<std::list<Match> (nodeid_t)> nodeAnnoMatchGenerator;

  std::list<Match> currentMatchBuffer;

private:
  bool nextMatchBuffer();
};

}

