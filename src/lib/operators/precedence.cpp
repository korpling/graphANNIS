#include "precedence.h"
#include "defaultjoins.h"

using namespace annis;

LegacyPrecedence::LegacyPrecedence(DB &db,
                       std::shared_ptr<AnnoIt> left, std::shared_ptr<AnnoIt> right,
                       unsigned int minDistance, unsigned int maxDistance)
  : db(db), tokHelper(db),
    left(left), right(right), minDistance(minDistance), maxDistance(maxDistance),
    tokIteratorForLeftNode(std::shared_ptr<RightMostTokenForNodeIterator>(new RightMostTokenForNodeIterator(left, db))),
    annoForRightNode(right->getAnnotation()),
    actualJoin(NULL),
    edbLeft(NULL), edbOrder(NULL),
    tokenShortcut(false)
{
  edbOrder = db.getEdgeDB(ComponentType::ORDERING, annis_ns, "");
  edbLeft = db.getEdgeDB(ComponentType::LEFT_TOKEN, annis_ns, "");
  if(edbOrder != NULL)
  {
    Annotation anyTokAnno = Init::initAnnotation(db.getTokStringID(), 0, db.getNamespaceStringID());
    Annotation anyNodeAnno = Init::initAnnotation(db.getNodeNameStringID(), 0, db.getNamespaceStringID());

    if(checkAnnotationEqual(left->getAnnotation(), anyTokAnno)
       && checkAnnotationEqual(right->getAnnotation(), anyTokAnno))
    {
      tokenShortcut = true;
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

LegacyPrecedence::~LegacyPrecedence()
{
  delete actualJoin;
}

void LegacyPrecedence::init(std::shared_ptr<AnnoIt> lhs, std::shared_ptr<AnnoIt> rhs)
{
  left = lhs;
  right = rhs;
}

BinaryMatch LegacyPrecedence::next()
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
        std::vector<nodeid_t> matchCandidateNodes;
        if(!tokenShortcut)
        {
          matchCandidateNodes = edbLeft->getOutgoingEdges(currentMatchedToken.rhs.node);
        }
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
      result.lhs = tokIteratorForLeftNode->currentNodeMatch();
      result.rhs = currentMatches.top();
      currentMatches.pop();
      return result;
    }

  } // end if join and edge db for left_token component initialized
  return result;
}

void LegacyPrecedence::reset()
{
  if(actualJoin != nullptr)
  {
    actualJoin->reset();
  }
  while(!currentMatches.empty())
  {
    currentMatches.pop();
  }
  currentMatchedToken.found = true;
}


Precedence::Precedence(const DB &db, unsigned int minDistance, unsigned int maxDistance)
  : tokHelper(db), edbOrder(db.getEdgeDB(ComponentType::ORDERING, annis_ns, "")),
    anyTokAnno(Init::initAnnotation(db.getTokStringID(), 0, db.getNamespaceStringID())),
    anyNodeAnno(Init::initAnnotation(db.getNodeNameStringID(), 0, db.getNamespaceStringID())),
    minDistance(minDistance), maxDistance(maxDistance)
{
}

std::unique_ptr<AnnoIt> Precedence::retrieveMatches(const Match &lhs)
{
  EdgeIterator* edgeIterator = edbOrder->findConnected(lhs.node, minDistance, maxDistance);
  if(checkAnnotationEqual(lhs.anno, anyTokAnno))
  {
    // special case: order relations always have token as target if the source is a token

  }
  else
  {
  }
}

bool Precedence::filter(const Match &lhs, const Match &rhs)
{
  nodeid_t lhsToken = tokHelper.rightTokenForNode(lhs.node);
  nodeid_t rhsToken = tokHelper.leftTokenForNode(rhs.node);
  if(edbOrder->isConnected(Init::initEdge(lhsToken, rhsToken), minDistance, maxDistance))
  {
    return true;
  }
  return false;
}

Precedence::~Precedence()
{

}
