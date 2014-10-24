#include "nestedloopjoin.h"

using namespace annis;

NestedLoopJoin::NestedLoopJoin(const EdgeDB *edb, AnnotationIterator& left, AnnotationIterator& right, unsigned int minDistance, unsigned int maxDistance)
  : edb(edb), left(left), right(right), minDistance(minDistance), maxDistance(maxDistance)
{
}


BinaryMatch NestedLoopJoin::next()
{
  BinaryMatch result;
  result.found = false;


  while(left.hasNext())
  {
    matchLeft = left.next();
    right.reset();

    while(right.hasNext())
    {
      matchRight = right.next();

      // check the actual constraint
      if(edb->isConnected(initEdge(matchLeft.first, matchRight.first), minDistance, maxDistance))
      {
        result.found = true;
        result.left = matchLeft;
        result.right = matchRight;

        // immediatly return
        return result;
      }

    }
  }
  return result;
}

void NestedLoopJoin::reset()
{
  left.reset();
  right.reset();
}

NestedLoopJoin::~NestedLoopJoin()
{

}

