#pragma once

#include <annis/types.h>
#include <annis/iterators.h>

#include <annis/util/sharedqueue.h>

#include <boost/lockfree/queue.hpp>
#include <thread>

#include <list>

namespace annis
{

class Operator;


class IndexJoin : public Iterator
{
public:
  struct MatchPair
  {
    Match lhs;
    Match rhs;
  };


public:
  IndexJoin(std::shared_ptr<Iterator> lhs, size_t lhsIdx,
            bool (*nextMatchFunc)(const Match &, Match &));

  virtual bool next(std::vector<Match>& tuple) override;
  virtual void reset() override;

  virtual ~IndexJoin() {}
private:
  SharedQueue<std::vector<Match>> results;
  bool fetchLoopStarted;
  std::function<void()> lhsFetchLoop;

  std::thread lhsFetcher;
};
}

