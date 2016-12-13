#pragma once

#include <annis/types.h>
#include <annis/graphstorage/graphstorage.h>
#include <annis/db.h>
#include <annis/iterators.h>

#include <ThreadPool.h>

namespace annis 
{
  class Operator;


  /** 
   * A join that checks all combinations of the left and right matches if their are connected. 
   * 
   * @param lhsIdx the column of the LHS tuple to join on
   * @param rhsIdx the column of the RHS tuple to join on
   */
  class NestedLoopJoin : public Iterator
  {
    struct MatchPair
    {
      std::vector<Match> lhs;
      Match rhs;
    };

  public:
    NestedLoopJoin(std::shared_ptr<Operator> op,
      std::shared_ptr<Iterator> lhs, std::shared_ptr<Iterator> rhs,
      size_t lhsIdx, size_t rhsIdx,
      bool leftIsOuter=true,
      unsigned maxBufferedTasks = std::thread::hardware_concurrency(),
      std::shared_ptr<ThreadPool> threadPool=std::shared_ptr<ThreadPool>());
    virtual ~NestedLoopJoin();

    virtual bool next(std::vector<Match>& tuple) override;
    virtual void reset() override;
  private:
    std::shared_ptr<Operator> op;

    const bool leftIsOuter;
    bool initialized;
    
    const unsigned maxBufferedTasks;
    std::shared_ptr<ThreadPool> threadPool;

    std::deque<std::future<std::deque<MatchPair>>> taskBuffer;
    std::deque<MatchPair> matchBuffer;

    std::vector<Match> matchOuter;
    std::vector<Match> matchInner;

    std::shared_ptr<Iterator> outer;
    std::shared_ptr<Iterator> inner;
    
    const size_t outerIdx;
    const size_t innerIdx;
    
    bool firstOuterFinished;
    std::list<std::vector<Match>> innerCache;
    std::list<std::vector<Match>>::const_iterator itInnerCache;
  private:
    bool fetchNextInner();
    void fillTaskBuffer();

  };


} // end namespace annis

