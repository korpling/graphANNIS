#include <annis/filter.h>

using namespace annis;


Filter::Filter(std::shared_ptr<Operator> op, std::shared_ptr<Iterator> inner,
  size_t lhsIdx, size_t rhsIdx)
  : op(op), inner(inner), lhsIdx(lhsIdx), rhsIdx(rhsIdx)
{

}

// TODO: explicitly test the filter function
bool Filter::next(std::vector<Match>& tuple)
{
  tuple.clear();
  bool found = false;

  if(op && inner)
  {
    std::vector<Match> innerMatch;
    while(!found && inner->next(innerMatch))
    {
      if(op->filter(innerMatch[lhsIdx], innerMatch[rhsIdx]))
      {
        tuple.reserve(innerMatch.size());
        tuple.insert(tuple.end(), innerMatch.begin(), innerMatch.end());
        found = true;
      }
    }
  }

  return found;
}

void Filter::reset()
{
  if(inner)
  {
    inner->reset();
  }
}

Filter::~Filter()
{

}
