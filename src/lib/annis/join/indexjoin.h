#pragma once

#include <annis/types.h>
#include <annis/iterators.h>

#include <annis/util/sharedqueue.h>
#include <annis/util/comparefunctions.h>

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
            std::shared_ptr<Operator> op,
            std::function<std::list<Match> (nodeid_t)> matchGeneratorFunc);

  virtual bool next(std::vector<Match>& tuple) override;
  virtual void reset() override;

  virtual ~IndexJoin();
private:
  bool fetchLoopStarted;


  SharedQueue<std::vector<Match>> results;
  std::function<void()> lhsFetchLoop;

  std::thread lhsFetcher;

private:

};
}

