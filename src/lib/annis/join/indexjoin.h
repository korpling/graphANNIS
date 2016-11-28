#pragma once

#include <annis/types.h>
#include <annis/iterators.h>

#include <boost/lockfree/queue.hpp>
#include <thread>

#include <list>

namespace annis
{

class Operator;


class IndexJoin : public Iterator
{
public:
  enum class State {INIT, STARTED, FINISHED};

  struct MatchPair
  {
    Match lhs;
    Match rhs;
    size_t lhsIndex;
    size_t rhsIndex;
  };


public:
  IndexJoin(std::shared_ptr<Iterator> lhs, size_t lhsIdx,
            Match (*nextMatchFunc)(const Match& lhs));

  virtual bool next(std::vector<Match>& tuple) override;
  virtual void reset() override;

  virtual ~IndexJoin() {}
private:
  boost::lockfree::queue<MatchPair, boost::lockfree::capacity<8>> results;

  std::shared_ptr<Iterator> lhs;
  const size_t lhsIdx;

  Match (*nextMatchFunc)(const Match& lhs);

  State execState;

private:
  void lhsFetchLoop();
};
}

