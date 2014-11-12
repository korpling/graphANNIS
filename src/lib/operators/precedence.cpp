#include "precedence.h"
#include "defaultjoins.h"

using namespace annis;

Precedence::Precedence(DB &db, AnnotationIterator& left, AnnotationIterator& right,
                       unsigned int minDistance, unsigned int maxDistance)
  : db(db), left(left), right(right), minDistance(minDistance), maxDistance(maxDistance),
    tokIteratorForLeftNode(RightMostTokenForNodeIterator(left, db)),
    annoForRightNode(right.getAnnotation()),
    actualJoin(NULL),
    edbLeft(NULL)
{
  const EdgeDB* edbOrder = db.getEdgeDB(ComponentType::ORDERING, annis_ns, "");
  edbLeft = db.getEdgeDB(ComponentType::LEFT_TOKEN, annis_ns, "");
  if(edbOrder != NULL)
  {
    Annotation anyTokAnno = initAnnotation(db.getTokStringID(), 0, db.getNamespaceStringID());
    // TODO: allow to use a nested loop iterator as a configurable alternative
    actualJoin = new SeedJoin(db, edbOrder, tokIteratorForLeftNode, anyTokAnno, minDistance, maxDistance);
  }
}

Precedence::~Precedence()
{
  delete actualJoin;
}

BinaryMatch Precedence::next()
{
  BinaryMatch result;
  result.found = false;
  if(actualJoin != NULL && edbLeft != NULL)
  {
    do
    {
      if(currentNodeCandiates.empty())
      {
        currentMatchedToken = actualJoin->next();
        if(currentMatchedToken.found)
        {
          currentNodeCandiates = edbLeft->getOutgoingEdges(currentMatchedToken.rhs.node);
          // also check the token itself
          currentNodeCandiates.insert(currentNodeCandiates.end(),
                                      currentMatchedToken.rhs.node);
        }
      }
      else
      {
        nodeid_t nodeID = currentNodeCandiates.back();
        currentNodeCandiates.pop_back();
        for(auto& nodeAnno : db.getNodeAnnotationsByID(nodeID))
        {
          if(checkAnnotationEqual(nodeAnno, annoForRightNode))
          {
            result.found = true;
            result.lhs = tokIteratorForLeftNode.currentNodeMatch();
            result.rhs.node = nodeID;
            result.rhs.anno = nodeAnno;
            return result;
          }
        }
      }

    } while(currentMatchedToken.found  || !currentNodeCandiates.empty());
  }
  return result;
}

void Precedence::reset()
{
  if(actualJoin != NULL)
  {
    actualJoin->reset();
  }
  currentNodeCandiates.clear();
}


RightMostTokenForNodeIterator::RightMostTokenForNodeIterator(AnnotationIterator &source, const DB &db)
  : source(source), db(db), edb(db.getEdgeDB(ComponentType::RIGHT_TOKEN, annis_ns, ""))
{
  anyTokAnnotation = initAnnotation(db.getTokStringID(), 0, db.getNamespaceStringID());
}

bool RightMostTokenForNodeIterator::hasNext()
{
  return source.hasNext();
}

Match RightMostTokenForNodeIterator::next()
{
  Match result;
  if(source.hasNext() && edb != NULL)
  {
    currentOriginalMatch = source.next();

    // check if this is a token
    std::vector<Annotation> annos = db.getNodeAnnotationsByID(currentOriginalMatch.node);
    for(auto& a : annos)
    {
      if(checkAnnotationEqual(anyTokAnnotation, a))
      {
        return currentOriginalMatch;
      }
    }

    result.node = edb->getOutgoingEdges(currentOriginalMatch.node)[0];
    result.anno.name = db.getTokStringID();
    result.anno.ns = db.getNamespaceStringID();
    result.anno.val = 0; //TODO: do we want to include the actual value here?
  }

  return result;
}

void RightMostTokenForNodeIterator::reset()
{
  source.reset();
}

Match RightMostTokenForNodeIterator::currentNodeMatch()
{

}

