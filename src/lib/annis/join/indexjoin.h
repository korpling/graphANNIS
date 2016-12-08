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


public:
  IndexJoin(std::shared_ptr<Iterator> lhs, size_t lhsIdx,
            std::shared_ptr<Operator> op,
            std::function<std::list<Match> (nodeid_t)> matchGeneratorFunc,
            unsigned maxNumfOfTasks = std::thread::hardware_concurrency());

  virtual bool next(std::vector<Match>& tuple) override;
  virtual void reset() override;

  virtual ~IndexJoin();
private:

  std::shared_ptr<Iterator> lhs;
  const size_t lhsIdx;
  const unsigned maxNumfOfTasks;

  std::list<std::future<std::list<MatchPair>>> taskBuffer;
  std::list<MatchPair> matchBuffer;


  std::function<std::list<MatchPair>(std::vector<Match>)> taskBufferGenerator;

private:
  void fillTaskBuffer();
  bool nextMatchBuffer();
};
}

