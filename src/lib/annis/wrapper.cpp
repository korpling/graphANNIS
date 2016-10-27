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
    std::vector<Match> tuple;
    if(wrappedJoin->next(tuple))
    {
      // add the match to this list *and* to the other one which is hold by the JoinWrapIterator
      if(wrapLeftOperand)
      {
        addMatch(tuple[lhsIdx]);
        if(!otherInnerWrapper.expired())
        {
          otherInnerWrapper.lock()->addMatch(tuple[rhsIdx]);
        }
      }
      else
      {
        addMatch(tuple[rhsIdx]);
        if(!otherInnerWrapper.expired())
        {
          otherInnerWrapper.lock()->addMatch(tuple[lhsIdx]);
        }
      }
    }
  }
}
