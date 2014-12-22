#include "query.h"
#include "operators/defaultjoins.h"

using namespace annis;

Query::Query()
{
}


size_t annis::Query::addNode(std::shared_ptr<annis::AnnotationIterator> n)
{
  size_t idx = source.size();
  source.push_back(n);
  return idx;
}

void Query::executeOperator(std::shared_ptr<BinaryOperatorIterator> op, size_t idxLeft, size_t idxRight)
{
  if(idxLeft < source.size() && idxRight < source.size())
  {
    op->init(source[idxLeft], source[idxRight]);
    source[idxLeft] = std::make_shared<JoinWrapIterator>(op, true);
    source[idxRight] = std::make_shared<JoinWrapIterator>(op, false);
  }
}

