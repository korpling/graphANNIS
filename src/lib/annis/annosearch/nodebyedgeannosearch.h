#pragma once

#include <annis/annosearch/annotationsearch.h>
#include <annis/graphstorage/graphstorage.h>
#include <functional>

namespace annis
{
class NodeByEdgeAnnoSearch : public EstimatedSearch
{

  using ItType = BTreeMultiAnnoStorage<Edge>::InverseAnnoMap_t::const_iterator;
  using Range = std::pair<ItType, ItType>;

public:
  NodeByEdgeAnnoSearch(std::vector<std::shared_ptr<const ReadableGraphStorage>> gs, std::set<Annotation> validEdgeAnnos,
                       std::function<std::list<Match> (nodeid_t)> nodeAnnoMatchGenerator, std::int64_t wrappedNodeCountEstimate,
                       std::string debugDescription="");

  virtual bool next(Match& m) override;
  virtual void reset() override;

  virtual std::int64_t guessMaxCount() const override {return wrappedNodeCountEstimate;}

  virtual std::string debugString() const override {return debugDescription;}

  virtual ~NodeByEdgeAnnoSearch();
private:
  std::function<std::list<Match> (nodeid_t)> nodeAnnoMatchGenerator;
  const std::int64_t wrappedNodeCountEstimate;
  const std::string debugDescription;


  std::list<Match> currentMatchBuffer;

  std::list<Range> searchRanges;
  std::list<Range>::const_iterator currentRange;
  ItType it;


private:
  bool nextMatchBuffer();
};

}

