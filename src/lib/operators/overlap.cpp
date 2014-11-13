#include "overlap.h"

using namespace annis;

Overlap::Overlap(DB &db, AnnotationIterator &left, AnnotationIterator &right)
  : left(left), right(right), db(db),
    edbLeft(db.getEdgeDB(ComponentType::LEFT_TOKEN, annis_ns, "")),
    edbRight(db.getEdgeDB(ComponentType::RIGHT_TOKEN, annis_ns, "")),
    edbOrder(db.getEdgeDB(ComponentType::ORDERING, annis_ns, ""))
    //lhsLeftTokenIt(left, db)
  //  tokenRightFromLHSIt(db, edbOrder, lhsLeftTokenIt, initAnnotation(db.getNodeNameStringID(), 0, db.getNamespaceStringID()), 0, uintmax)
{
  reset();
}

BinaryMatch Overlap::next()
{
  BinaryMatch result;
  result.found = false;


  while(left.hasNext())
  {
    result.lhs = left.next();
    nodeid_t lhsLeftToken = leftTokenForNode(result.lhs.node);
    nodeid_t lhsRightToken = rightTokenForNode(result.lhs.node);

    while(right.hasNext())
    {
      result.rhs = right.next();

      // get the left- and right-most covered token for rhs
      nodeid_t rhsLeftToken = leftTokenForNode(result.rhs.node);
      nodeid_t rhsRightToken = rightTokenForNode(result.rhs.node);
      if(edbOrder->isConnected(initEdge(lhsLeftToken, rhsRightToken), 0, uintmax) &&
        edbOrder->isConnected(initEdge(rhsLeftToken, lhsRightToken), 0, uintmax))
      {
         result.found = true;
         return result;
      }

    }

    right.reset();
  }

  return result;
}

void Overlap::reset()
{
  uniqueMatches.clear();
  left.reset();
  right.reset();
  //currentMatches.clear();
  //hsLeftTokenIt.reset();
  //tokenRightFromLHSIt.reset();
}

Overlap::~Overlap()
{

}

nodeid_t Overlap::leftTokenForNode(nodeid_t n)
{
  if(isToken(n))
  {
    return n;
  }
  else
  {
    return edbLeft->getOutgoingEdges(n)[0];
  }
}

nodeid_t Overlap::rightTokenForNode(nodeid_t n)
{
  if(isToken(n))
  {
    return n;
  }
  else
  {
    return edbRight->getOutgoingEdges(n)[0];
  }
}

bool Overlap::isToken(nodeid_t n)
{
  for(const Annotation& anno: db.getNodeAnnotationsByID(n))
  {
    if(anno.ns == db.getNamespaceStringID() && anno.name == db.getTokStringID())
    {
      // rhs is token by itself
      return true;
    }
  }
  return false;
}

