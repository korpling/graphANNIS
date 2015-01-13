#include "filter.h"

using namespace annis;


Filter::Filter(std::shared_ptr<Operator> op, std::shared_ptr<AnnoIt> lhs, std::shared_ptr<AnnoIt> rhs)
  : op(op), lhs(lhs), rhs(rhs)
{

}

BinaryMatch Filter::next()
{
  BinaryMatch result;
  result.found = false;

  if(op && lhs && rhs)
  {
    while(!result.found && lhs->hasNext() && rhs->hasNext())
    {
      result.lhs = lhs->next();
      result.rhs = rhs->next();

      if(op->filter(result.lhs, result.rhs))
      {
        result.found = true;
      }
    }
  }

  return result;
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
