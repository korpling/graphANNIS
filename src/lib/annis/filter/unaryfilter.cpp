#include "unaryfilter.h"

using namespace annis;

UnaryFilter::UnaryFilter(std::shared_ptr<AnnoIt> delegate, std::function<bool(const Match &)> filterFunc)
  : delegate(delegate), filterFunc(filterFunc)
{

}

bool UnaryFilter::next(Match &m)
{
  while(delegate->next(m))
  {
    // apply additional filter
    if(filterFunc(m))
    {
      return true;
    }
  }
  return false;
}

void UnaryFilter::reset()
{
  delegate->reset();
}
