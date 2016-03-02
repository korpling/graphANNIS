#include <annis/join/nestedloop.h>
#include <annis/annosearch/annotationsearch.h>
#include <annis/iterators.h>
#include <annis/operators/operator.h>

using namespace annis;


NestedLoopJoin::NestedLoopJoin(std::shared_ptr<Operator> op,
                               std::shared_ptr<AnnoIt> lhs,
                               std::shared_ptr<AnnoIt> rhs,
                               bool leftIsOuter)
  : op(op), leftIsOuter(leftIsOuter), initialized(false),
    outer(leftIsOuter ? lhs : rhs), inner(leftIsOuter ? rhs : lhs)
{
}

bool NestedLoopJoin::next(Match& lhsMatch, Match& rhsMatch)
{

  if(!op || !outer || !inner)
  {
    return false;
  }

  bool proceed = true;

  if(!initialized)
  {
    proceed = false;
    if(outer->hasNext())
    {
      matchOuter = outer->next();
      proceed = true;
      initialized = true;
    }
  }

  while(proceed)
  {

    while(inner->hasNext())
    {
      matchInner = inner->next();

      bool include = true;
      // do not include the same match if not reflexive
      if(!op->isReflexive()
         && matchOuter.node == matchInner.node
         && checkAnnotationKeyEqual(matchOuter.anno, matchInner.anno)) {
        include = false;
      }
      
      if(include)
      {
        if(leftIsOuter)
        {
          if(op->filter(matchOuter, matchInner))
          {
            lhsMatch = matchOuter;
            rhsMatch = matchInner;

            return true;
          }
        }
        else
        {
          if(op->filter(matchInner, matchOuter))
          {
            lhsMatch = matchInner;
            rhsMatch = matchOuter;

            return true;
          }
        }
      } // end if include

    } // end for each right

    if(outer->hasNext())
    {
      matchOuter = outer->next();
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
