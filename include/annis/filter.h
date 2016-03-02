#pragma once

#include <annis/iterators.h>
#include <annis/operators/operator.h>

namespace annis
{

class Filter : public Iterator
{
public:

  Filter(std::shared_ptr<Operator> op, std::shared_ptr<AnnoIt> lhs, std::shared_ptr<AnnoIt> rhs);

  virtual bool next(Match& lhsMatch, Match& rhsMatch) override;
  virtual void reset() override;

  virtual ~Filter();

private:
  std::shared_ptr<Operator> op;
  std::shared_ptr<AnnoIt> lhs;
  std::shared_ptr<AnnoIt> rhs;
};

} // end namespace annis
