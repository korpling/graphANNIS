#pragma once

#include <annis/types.h>
#include <annis/graphstorage/graphstorage.h>
#include <annis/db.h>

namespace annis 
{
  class Operator;
  class AnnoIt;
  class Iterator;

  /** A join that checks all combinations of the left and right matches if their are connected. */
  class NestedLoopJoin : public Iterator
  {
  public:
    NestedLoopJoin(std::shared_ptr<Operator> op,
      std::shared_ptr<AnnoIt> lhs, std::shared_ptr<AnnoIt> rhs, bool leftIsOuter=true);
    virtual ~NestedLoopJoin();

    virtual bool next(Match& lhsMatch, Match& rhsMatch) override;
    virtual void reset() override;
  private:
    std::shared_ptr<Operator> op;
    bool initialized;
    bool leftIsOuter;

    std::shared_ptr<AnnoIt> outer;
    std::shared_ptr<AnnoIt> inner;

    Match matchOuter;
    Match matchInner;

  };


} // end namespace annis

