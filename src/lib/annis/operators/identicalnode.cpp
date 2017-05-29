#include "identicalnode.h"

#include <annis/wrapper.h>

using namespace annis;

IdenticalNode::IdenticalNode(const DB& db)
  : anyNodeAnno({db.getNodeNameStringID(), db.getNamespaceStringID(), 0})
{
}


IdenticalNode::~IdenticalNode()
{

}

std::unique_ptr<AnnoIt> IdenticalNode::retrieveMatches(const Match &lhs)
{
  // just return the node itself
  Match m;
  m.node = lhs.node;
  m.anno = anyNodeAnno;
  return std::unique_ptr<AnnoIt>(new SingleElementWrapper(m));
}

bool IdenticalNode::filter(const Match &lhs, const Match &rhs)
{
  return lhs.node == rhs.node;
}

