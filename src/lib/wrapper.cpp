#include <annis/wrapper.h>



using namespace annis;

ListWrapper::ListWrapper()
{
}


ListWrapper::~ListWrapper()
{
  
}

void JoinWrapIterator::reset()
{
  // reset all internal state
  ListWrapper::reset();
  if(!otherInnerWrapper.expired())
  {
    otherInnerWrapper.lock()->ListWrapper::reset();
  }
  // also reset the actual join operator
  if(wrappedJoin)
  {
    wrappedJoin->reset();
  }
}

void JoinWrapIterator::checkIfNextCallNeeded()
{
  // if the current list of entries is empty call the underlying join
  bool isEmpty = internalEmpty();
  bool joinIsValid = (bool) wrappedJoin;
  if(isEmpty && joinIsValid)
  {
    Match nextLHS;
    Match nextRHS;
    if(wrappedJoin->next(nextLHS, nextRHS))
    {
      // add the match to this list *and* to the other one which is hold by the JoinWrapIterator
      if(wrapLeftOperand)
      {
        addMatch(nextLHS);
        if(!otherInnerWrapper.expired())
        {
          otherInnerWrapper.lock()->addMatch(nextRHS);
        }
      }
      else
      {
        addMatch(nextRHS);
        if(!otherInnerWrapper.expired())
        {
          otherInnerWrapper.lock()->addMatch(nextLHS);
        }
      }
    }
  }
}
