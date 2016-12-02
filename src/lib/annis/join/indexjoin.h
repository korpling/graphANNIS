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
  struct MatchCandidate
  {
    bool valid;
    std::list<Match> rhs;
  };


public:
  IndexJoin(std::shared_ptr<Iterator> lhs, size_t lhsIdx,
            std::shared_ptr<Operator> op,
            std::function<std::list<Match> (nodeid_t)> matchGeneratorFunc);

  virtual bool next(std::vector<Match>& tuple) override;
  virtual void reset() override;

  virtual ~IndexJoin();
private:

  std::shared_ptr<Iterator> lhs;
  const size_t lhsIdx;
  std::shared_ptr<Operator> op;

  std::vector<Match> currentLHS;

  std::queue<std::future<MatchCandidate>> rhsBuffer;

  std::queue<Match> rhsAnnoBuffer;

  std::function<MatchCandidate(const Match&, nodeid_t)> rhsBufferGenerator;

private:
  bool fetchNextLHS();
};
}

