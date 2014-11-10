#include "componentchainjoin.h"

#include "defaultjoins.h"
using namespace annis;

ComponentChainJoin::ComponentChainJoin(AnnotationIterator& lhs, AnnotationIterator& rhs,
                                       const std::list<ComponentChainEntry> &entries)
  :lhs(lhs), rhs(rhs), entries(entries)
{
  reset();
}

BinaryMatch ComponentChainJoin::next()
{
  BinaryMatch result;
  result.found = false;
  const ComponentChainEntry& e = *itEntries;
  // TODO: implement

  return result;
}

void ComponentChainJoin::reset()
{
  itEntries = entries.begin();
}

ComponentChainJoin::~ComponentChainJoin()
{

}
