#include "wrapper.h"

using namespace annis;

void JoinWrapIterator::reset()
{
  ListWrapper::reset();
  if(!otherInnerWrapper.expired())
  {
    otherInnerWrapper.lock()->reset();
  }
}

void JoinWrapIterator::checkIfNextCallNeeded()
{
  // if the current list of entries is empty call the underlying join
  if(internalEmpty() && wrappedJoin)
  {
    BinaryMatch nextMatch = wrappedJoin->next();
    if(nextMatch.found)
    {
      // add the match to this list *and* to the other one which is hold by the JoinWrapIterator
      if(wrapLeftOperand)
      {
        addMatch(nextMatch.lhs);
        if(!otherInnerWrapper.expired())
        {
          otherInnerWrapper.lock()->addMatch(nextMatch.rhs);
        }
      }
      else
      {
        addMatch(nextMatch.rhs);
        if(!otherInnerWrapper.expired())
        {
          otherInnerWrapper.lock()->addMatch(nextMatch.lhs);
        }
      }
    }
  }
}
