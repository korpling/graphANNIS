#pragma once

#include <annis/types.h>
#include <annis/graphstorage/graphstorage.h>
#include <annis/db.h>

namespace annis 
{
  class Operator;
  class AnnoIt;
  class Iterator;

  /** 
   * A join that checks all combinations of the left and right matches if their are connected. 
   * 
   * @param lhsIdx the column of the LHS tuple to join on
   * @param rhsIdx the column of the RHS tuple to join on
   */
  class NestedLoopJoin : public Iterator
  {
  public:
    NestedLoopJoin(std::shared_ptr<Operator> op,
      std::shared_ptr<Iterator> lhs, std::shared_ptr<Iterator> rhs,
      size_t lhsIdx, size_t rhsIdx,
      bool materializeInner=true,
      bool leftIsOuter=true);
    virtual ~NestedLoopJoin();

    virtual bool next(std::vector<Match>& tuple) override;
    virtual void reset() override;
  private:
    std::shared_ptr<Operator> op;

    const bool materializeInner;
    const bool leftIsOuter;
    bool initialized;
    
    std::vector<Match> matchOuter;
    std::vector<Match> matchInner;

    std::shared_ptr<Iterator> outer;
    std::shared_ptr<Iterator> inner;
    
    size_t outerIdx;
    size_t innerIdx;
    
    bool firstOuterFinished;
    std::list<std::vector<Match>> innerCache;
    std::list<std::vector<Match>>::const_iterator itInnerCache;
  private:
    bool fetchNextInner();

  };


} // end namespace annis

