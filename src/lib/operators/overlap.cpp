#include "overlap.h"

using namespace annis;

NestedOverlap::NestedOverlap(DB &db, AnnotationIterator &left, AnnotationIterator &right)
  : left(left), right(right), db(db),
    edbLeft(db.getEdgeDB(ComponentType::LEFT_TOKEN, annis_ns, "")),
    edbRight(db.getEdgeDB(ComponentType::RIGHT_TOKEN, annis_ns, "")),
    edbOrder(db.getEdgeDB(ComponentType::ORDERING, annis_ns, ""))
    //lhsLeftTokenIt(left, db)
  //  tokenRightFromLHSIt(db, edbOrder, lhsLeftTokenIt, initAnnotation(db.getNodeNameStringID(), 0, db.getNamespaceStringID()), 0, uintmax)
{
  reset();
}

BinaryMatch NestedOverlap::next()
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

void NestedOverlap::reset()
{
  uniqueMatches.clear();
  left.reset();
  right.reset();
  //currentMatches.clear();
  //hsLeftTokenIt.reset();
  //tokenRightFromLHSIt.reset();
}

NestedOverlap::~NestedOverlap()
{

}

nodeid_t NestedOverlap::leftTokenForNode(nodeid_t n)
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

nodeid_t NestedOverlap::rightTokenForNode(nodeid_t n)
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

bool NestedOverlap::isToken(nodeid_t n)
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

SeedOverlap::SeedOverlap(DB &db, AnnotationIterator &left, AnnotationIterator &right)
  : left(left), rightAnnotation(right.getAnnotation()), db(db),
    edbLeft(db.getEdgeDB(ComponentType::LEFT_TOKEN, annis_ns, "")),
    edbRight(db.getEdgeDB(ComponentType::RIGHT_TOKEN, annis_ns, "")),
    edbOrder(db.getEdgeDB(ComponentType::ORDERING, annis_ns, "")),
    lhsLeftTokenIt(left, db),
    tokenRightFromLHSIt(db, edbOrder, lhsLeftTokenIt, initAnnotation(db.getNodeNameStringID(), 0, db.getNamespaceStringID()), 0, uintmax)
{
  reset();
}

BinaryMatch SeedOverlap::next()
{
  BinaryMatch result;
  result.found = false;

  BinaryMatch rightTokenMatch;

  if(currentMatches.empty())
  {
    rightTokenMatch = tokenRightFromLHSIt.next();
  }
  else
  {
    rightTokenMatch.found = false;
  }
  while(currentMatches.empty() && rightTokenMatch.found)
  {
    result.lhs = lhsLeftTokenIt.currentNodeMatch();

    // get the node that has a right border with the token
    std::vector<nodeid_t> overlapCandidates = edbRight->getOutgoingEdges(rightTokenMatch.rhs.node);
    // also add the token itself
    overlapCandidates.insert(overlapCandidates.begin(), rightTokenMatch.rhs.node);

    // check each candidate if it's left side comes before the right side of the lhs node
    for(unsigned int i=0; i < overlapCandidates.size(); i++)
    {
      nodeid_t candidateID = overlapCandidates[i];
      // the first candidate is always the token itself, otherwise get the aligned token
      nodeid_t leftTokenForCandidate = i == 0 ? candidateID : edbLeft->getOutgoingEdges(candidateID)[0];

      std::list<Annotation> matchingAnnos;
      for(const Annotation& anno : db.getNodeAnnotationsByID(candidateID))
      {
        if(checkAnnotationEqual(rightAnnotation, anno))
        {
          matchingAnnos.push_back(anno);
        }
      }

      if(!matchingAnnos.empty())
      {
        if(edbOrder->isConnected(initEdge(leftTokenForCandidate, rightTokenMatch.lhs.node), 0, uintmax))
        {
          Match m;
          m.node = candidateID;
          for(const Annotation& anno : matchingAnnos)
          {
            m.anno = anno;
            currentMatches.push_back(m);
          }
        }
      }
    }

    rightTokenMatch = tokenRightFromLHSIt.next();
  } // end while

  while(!currentMatches.empty())
  {
    result.found = true;
    result.rhs = currentMatches.front();
    currentMatches.pop_front();
  }

  return result;
}

void SeedOverlap::reset()
{
  uniqueMatches.clear();
  left.reset();
  currentMatches.clear();
  lhsLeftTokenIt.reset();
  tokenRightFromLHSIt.reset();
}

SeedOverlap::~SeedOverlap()
{

}

