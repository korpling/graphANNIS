#include "wrapper.h"


using namespace annis;


JoinWrapIterator::JoinWrapIterator(std::shared_ptr<BinaryIt> wrappedIterator, bool wrapLeftOperand)
  : matchAllAnnotation(Init::initAnnotation()), wrappedIterator(wrappedIterator), wrapLeftOperand(wrapLeftOperand)
{
  reset();
}


bool JoinWrapIterator::hasNext()
{
  return currentMatch.found;
}

Match JoinWrapIterator::next()
{
  Match result;
  if(currentMatch.found)
  {
    if(wrapLeftOperand)
    {
      result = currentMatch.lhs;
    }
    else
    {
      result = currentMatch.rhs;
    }
    currentMatch = wrappedIterator->next();
  }
  return result;
}

void JoinWrapIterator::reset()
{
  wrappedIterator->reset();
  currentMatch = wrappedIterator->next();
}

Match JoinWrapIterator::current()
{
  Match result;
  if(currentMatch.found)
  {
    if(wrapLeftOperand)
    {
      result = currentMatch.lhs;
    }
    else
    {
      result = currentMatch.rhs;
    }
  }
  return result;
}
