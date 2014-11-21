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
    Annotation anyTokAnno = Init::initAnnotation(db.getTokStringID(), 0, db.getNamespaceStringID());
    Annotation anyNodeAnno = Init::initAnnotation(db.getNodeNameStringID(), 0, db.getNamespaceStringID());

    if(checkAnnotationEqual(left.getAnnotation(), anyTokAnno)
       && checkAnnotationEqual(right.getAnnotation(), anyTokAnno))
    {
      // special case: order relations always have token as target if the source is a token
      actualJoin = new SeedJoin(db, edbOrder, tokIteratorForLeftNode, anyNodeAnno, minDistance, maxDistance);

    }
    else
    {
      // TODO: allow to use a nested loop iterator as a configurable alternative
      actualJoin = new SeedJoin(db, edbOrder, tokIteratorForLeftNode, anyTokAnno, minDistance, maxDistance);
    }
  }
  currentMatchedToken.found = true;
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
    while(currentMatches.empty() && currentMatchedToken.found)
    {
      currentMatchedToken = actualJoin->next();
      if(currentMatchedToken.found)
      {
        std::vector<nodeid_t> matchCandidateNodes = edbLeft->getOutgoingEdges(currentMatchedToken.rhs.node);
        // also check the token itself
        matchCandidateNodes.insert(matchCandidateNodes.end(),
                                   currentMatchedToken.rhs.node);

        for(const nodeid_t& nodeID : matchCandidateNodes)
        {
          for(const auto& nodeAnno : db.getNodeAnnotationsByID(nodeID))
          {
            if(checkAnnotationEqual(nodeAnno, annoForRightNode))
            {
              Match m;
              m.node = nodeID;
              m.anno = nodeAnno;
              currentMatches.push(m);
            }
          } // end for each annotation of the match candidate
        } // end for each match (rhs) candidate
      } // end if matched token found
    } // end while no current matches left and any token matched token found

    if(!currentMatches.empty())
    {
      result.found = true;
      result.lhs = tokIteratorForLeftNode.currentNodeMatch();
      result.rhs = currentMatches.top();
      currentMatches.pop();
      return result;
    }

  } // end if join and edge db for left_token component initialized
  return result;
}

void Precedence::reset()
{
  if(actualJoin != NULL)
  {
    actualJoin->reset();
  }
  while(!currentMatches.empty())
  {
    currentMatches.pop();
  }
  currentMatchedToken.found = true;
}

