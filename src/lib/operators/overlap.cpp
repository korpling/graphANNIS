#include "overlap.h"

using namespace annis;

Overlap::Overlap(DB &db, AnnotationIterator &left, AnnotationIterator &right)
  : left(left), rightAnnotation(right.getAnnotation()), db(db),
    edbLeft(db.getEdgeDB(ComponentType::LEFT_TOKEN, annis_ns, "")),
    edbRight(db.getEdgeDB(ComponentType::RIGHT_TOKEN, annis_ns, "")),
    edbOrder(db.getEdgeDB(ComponentType::ORDERING, annis_ns, "")),
    lhsLeftTokenIt(LeftMostTokenForNodeIterator(left, db)),
    tokenRightFromLHSIt(db, edbOrder, lhsLeftTokenIt, initAnnotation(db.getNodeNameStringID(), 0, db.getNamespaceStringID()), 0, uintmax)
{
  reset();
}

BinaryMatch Overlap::next()
{
  BinaryMatch result;
  result.found = false;

  // TODO: implement overlap
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

      if(edbOrder->isConnected(initEdge(leftTokenForCandidate, rightTokenMatch.rhs.node), 0, uintmax))
      {
        Match m;
        m.node = candidateID;
        for(const Annotation& anno : db.getNodeAnnotationsByID(candidateID))
        {
          if(checkAnnotationEqual(rightAnnotation, anno))
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

void Overlap::reset()
{
  uniqueMatches.clear();
  left.reset();
  currentMatches.clear();
  lhsLeftTokenIt.reset();
  tokenRightFromLHSIt.reset();
}

Overlap::~Overlap()
{

}

