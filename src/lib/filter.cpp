#include <annis/filter.h>

using namespace annis;


Filter::Filter(std::shared_ptr<Operator> op, std::shared_ptr<Iterator> lhs, std::shared_ptr<Iterator> rhs,
  size_t lhsIdx, size_t rhsIdx)
  : op(op), lhs(lhs), rhs(rhs), lhsIdx(lhsIdx), rhsIdx(rhsIdx)
{

}

bool Filter::next(std::vector<Match>& tuple)
{
  tuple.clear();
  bool found = false;

  if(op && lhs && rhs)
  {
    std::vector<Match> lhsMatch;
    std::vector<Match> rhsMatch;
    while(!found && lhs->next(lhsMatch) && rhs->next(rhsMatch))
    {
      if(op->filter(lhsMatch[lhsIdx], rhsMatch[rhsIdx]))
      {
        tuple.reserve(lhsMatch.size()+rhsMatch.size());
        tuple.insert(tuple.end(), lhsMatch.begin(), lhsMatch.end());
        tuple.insert(tuple.end(), rhsMatch.begin(), rhsMatch.end());
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
