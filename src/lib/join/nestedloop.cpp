#include <annis/join/nestedloop.h>
#include <annis/annosearch/annotationsearch.h>
#include <annis/iterators.h>
#include <annis/operators/operator.h>

using namespace annis;


NestedLoopJoin::NestedLoopJoin(std::shared_ptr<Operator> op,
                               std::shared_ptr<Iterator> lhs,
                               std::shared_ptr<Iterator> rhs,
                               size_t lhsIdx, size_t rhsIdx,
                               bool leftIsOuter)
  : op(op), leftIsOuter(leftIsOuter), initialized(false),
    outer(leftIsOuter ? lhs : rhs), inner(leftIsOuter ? rhs : lhs),
    outerIdx(leftIsOuter ? lhsIdx : rhsIdx), innerIdx(leftIsOuter ? rhsIdx : lhsIdx)
{
}

bool NestedLoopJoin::next(std::vector<Match>& result)
{
  result.clear();
  
  if(!op || !outer || !inner)
  {
    return false;
  }

  bool proceed = true;
  
  if(!initialized)
  {
    proceed = false;
    if(outer->next(matchOuter))
    {
      proceed = true;
      initialized = true;
    }
  }

  while(proceed)
  {

    while(inner->next(matchInner))
    {
      bool include = true;
      // do not include the same match if not reflexive
      if(!op->isReflexive()
         && matchOuter[outerIdx].node == matchInner[innerIdx].node
         && checkAnnotationKeyEqual(matchOuter[outerIdx].anno, matchInner[innerIdx].anno)) {
        include = false;
      }
      
      if(include)
      {
        if(leftIsOuter)
        {
          if(op->filter(matchOuter[outerIdx], matchInner[innerIdx]))
          {
            result.reserve(matchInner.size() + matchOuter.size());
            // return a tuple where the first values are from the outer relation and the iner relations tuples are added behind
            
            result.insert(result.end(), matchOuter.begin(), matchOuter.end());
            result.insert(result.end(), matchInner.begin(), matchInner.end());

            return true;
          }
        }
        else
        {
          if(op->filter(matchInner[innerIdx], matchOuter[outerIdx]))
          {
            result.reserve(matchInner.size() + matchOuter.size());
            // return a tuple where the first values are from the inner relation and the outer relations tuples are added behind
            result.insert(result.end(), matchInner.begin(), matchInner.end());
            result.insert(result.end(), matchOuter.begin(), matchOuter.end());
           

            return true;
          }
        }
      } // end if include

    } // end for each right

    if(outer->next(matchOuter))
    {
      inner->reset();
    }
    else
    {
      proceed = false;
    }
  } // end while proceed
  return false;
}

void NestedLoopJoin::reset()
{
  outer->reset();
  inner->reset();
  initialized = false;
}

NestedLoopJoin::~NestedLoopJoin()
{

}
