#include "indexjoin.h"


#include <annis/operators/operator.h>


using namespace annis;

IndexJoin::IndexJoin(std::shared_ptr<Iterator> lhs, size_t lhsIdx,
                     Match (*nextMatchFunc)(const Match &))
  : lhs(lhs), lhsIdx(lhsIdx), nextMatchFunc(nextMatchFunc)
{

}

bool IndexJoin::next(std::vector<Match> &tuple)
{
  return false;
}

void IndexJoin::reset()
{

}
