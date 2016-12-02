#pragma once

#include <annis/types.h>
#include <annis/iterators.h>

#include <annis/util/sharedqueue.h>
#include <annis/util/comparefunctions.h>

#include <boost/lockfree/queue.hpp>
#include <thread>
#include <future>

#include <list>

namespace annis
{

class Operator;


class IndexJoin : public Iterator
{
public:
  struct MatchPair
  {
    std::vector<Match> lhs;
    Match rhs;
  };


  struct ResultListEntry
  {
    MatchPair val;
    std::shared_future<std::shared_ptr<ResultListEntry>> next;
  };

  using RLENext = std::shared_future<std::shared_ptr<ResultListEntry>>;


public:
  IndexJoin(std::shared_ptr<Iterator> lhs, size_t lhsIdx,
            std::shared_ptr<Operator> op,
            std::function<std::list<Match> (nodeid_t)> matchGeneratorFunc);

  virtual bool next(std::vector<Match>& tuple) override;
  virtual void reset() override;

  virtual ~IndexJoin();
private:
  bool fetchLoopStarted;


  SharedQueue<std::vector<Match>> results;
  std::shared_ptr<Iterator> lhs;
  const size_t lhsIdx;
  std::shared_ptr<Operator> op;
  std::function<std::list<Match>(nodeid_t)> matchGeneratorFunc;

  RLENext resultFuture;

  std::function<void()> lhsFetchLoop;

  std::thread lhsFetcher;

private:

};
}

