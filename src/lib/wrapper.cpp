#include "wrapper.h"

using namespace annis;

void JoinWrapIterator::reset()
{
  ListWrapper::reset();
  if(otherInnerWrapper)
  {
    otherInnerWrapper->reset();
  }
}

void JoinWrapIterator::checkIfNextCallNeeded()
{
  // if the current list of entries is entry call the underlying join
  if(internalListSize() == 0)
  {
    BinaryMatch nextMatch = wrappedJoin->next();
    if(nextMatch.found)
    {
      // add the match to this list *and* to the other one which is hold by the JoinWrapIterator
      if(wrapLeftOperand)
      {
        addMatch(nextMatch.lhs);
        if(otherInnerWrapper)
        {
          otherInnerWrapper->addMatch(nextMatch.rhs);
        }
      }
      else
      {
        addMatch(nextMatch.rhs);
        if(otherInnerWrapper)
        {
          otherInnerWrapper->addMatch(nextMatch.lhs);
        }
      }
    }
  }
}
