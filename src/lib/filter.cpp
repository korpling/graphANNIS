#include <annis/filter.h>

using namespace annis;


Filter::Filter(std::shared_ptr<Operator> op, std::shared_ptr<AnnoIt> lhs, std::shared_ptr<AnnoIt> rhs)
  : op(op), lhs(lhs), rhs(rhs)
{

}

bool Filter::next(Match& lhsMatch, Match& rhsMatch)
{
  bool found = false;

  if(op && lhs && rhs)
  {
    while(!found && lhs->next(lhsMatch) && rhs->next(rhsMatch))
    {
      if(op->filter(lhsMatch, rhsMatch))
      {
        found = true;
      }
    }
  }

  return found;
}

void Filter::reset()
{
  if(lhs && rhs)
  {
    lhs->reset();
    rhs->reset();
  }
}

Filter::~Filter()
{

}
