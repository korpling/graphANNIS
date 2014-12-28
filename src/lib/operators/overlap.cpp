#include "overlap.h"

using namespace annis;

NestedOverlap::NestedOverlap(DB &db, std::shared_ptr<AnnoIt> left, std::shared_ptr<AnnoIt> right)
  : left(left), right(right), db(db),
    edbLeft(db.getEdgeDB(ComponentType::LEFT_TOKEN, annis_ns, "")),
    edbRight(db.getEdgeDB(ComponentType::RIGHT_TOKEN, annis_ns, "")),
    edbOrder(db.getEdgeDB(ComponentType::ORDERING, annis_ns, "")),
    initialized(false),
    tokenHelper(db)
    //lhsLeftTokenIt(left, db)
  //  tokenRightFromLHSIt(db, edbOrder, lhsLeftTokenIt, initAnnotation(db.getNodeNameStringID(), 0, db.getNamespaceStringID()), 0, uintmax)
{
  reset();
}

void NestedOverlap::init(std::shared_ptr<AnnoIt> lhs, std::shared_ptr<AnnoIt> rhs)
{
  left = lhs;
  right = rhs;
}

BinaryMatch NestedOverlap::next()
{

  BinaryMatch result;
  result.found = false;

  if(edbLeft == NULL || edbRight == NULL || edbOrder == NULL)
  {
    return result;
  }

  bool proceed = true;

  if(!initialized)
  {
    proceed = false;
    if(left->hasNext())
    {
      matchLHS = left->next();
      proceed = true;
      initialized = true;
    }
  }

  while(proceed)
  {

    nodeid_t lhsLeftToken = tokenHelper.leftTokenForNode(matchLHS.node);
    nodeid_t lhsRightToken = tokenHelper.rightTokenForNode(matchLHS.node);

    while(right->hasNext())
    {
      matchRHS = right->next();

      // get the left- and right-most covered token for rhs
      nodeid_t rhsLeftToken = tokenHelper.leftTokenForNode(matchRHS.node);
      nodeid_t rhsRightToken = tokenHelper.rightTokenForNode(matchRHS.node);


      // check the actual constraint
      if(edbOrder->isConnected(Init::initEdge(lhsLeftToken, rhsRightToken), 0, uintmax) &&
         edbOrder->isConnected(Init::initEdge(rhsLeftToken, lhsRightToken), 0, uintmax))
      {
        result.found = true;
        result.lhs = matchLHS;
        result.rhs = matchRHS;

        // immediatly return
        return result;
      }
    }
    if(left->hasNext())
    {
      matchLHS= left->next();
      right->reset();
    }
    else
    {
      proceed = false;
    }
  }
  return result;
}

void NestedOverlap::reset()
{
  //uniqueMatches.clear();
  left->reset();
  right->reset();
  //currentMatches.clear();
  //hsLeftTokenIt.reset();
  //tokenRightFromLHSIt.reset();
}

NestedOverlap::~NestedOverlap()
{

}

SeedOverlap::SeedOverlap(DB &db)
  :
    db(db),
    anyNodeAnno(Init::initAnnotation(db.getNodeNameStringID(), 0, db.getNamespaceStringID())),
    edbLeft(db.getEdgeDB(ComponentType::LEFT_TOKEN, annis_ns, "")),
    edbRight(db.getEdgeDB(ComponentType::RIGHT_TOKEN, annis_ns, "")),
    edbOrder(db.getEdgeDB(ComponentType::ORDERING, annis_ns, "")),
    edbCoverage(db.getEdgeDB(ComponentType::COVERAGE, annis_ns, "")),
    tokenCoveredByLHS(NULL)
//    lhsLeftTokenIt(left, db),
//    tokenRightFromLHSIt(db, edbOrder, lhsLeftTokenIt, initAnnotation(db.getNodeNameStringID(), 0, db.getNamespaceStringID()), 0, uintmax)
{
  reset();
}

void SeedOverlap::init(std::shared_ptr<AnnoIt> lhs, std::shared_ptr<AnnoIt> rhs)
{
  left = lhs;
  rightAnnotation = rhs->getAnnotation();
  tokenCoveredByLHS = new SeedJoin(db, edbCoverage, left, anyNodeAnno);
}

BinaryMatch SeedOverlap::next()
{
  BinaryMatch result;
  result.found = false;

  BinaryMatch coveredTokenMatch;
  if(currentMatches.empty())
  {
    coveredTokenMatch = tokenCoveredByLHS->next();
  }
  else
  {
    coveredTokenMatch.found = false;
  }


  while(currentMatches.empty() && coveredTokenMatch.found)
  {
    result.lhs = coveredTokenMatch.lhs;

    // get all nodes that are covering the token
    std::vector<nodeid_t> overlapCandidates = edbCoverage->getIncomingEdges(coveredTokenMatch.rhs.node);

     // also add the token itself
    overlapCandidates.push_back(coveredTokenMatch.rhs.node);

    // check the annotations for the candidates
    for(const nodeid_t& candidateID :  overlapCandidates)
    {
      for(const Annotation& anno : db.getNodeAnnotationsByID(candidateID))
      {
        if(checkAnnotationEqual(rightAnnotation, anno))
        {
          Match m;
          m.node = candidateID;
          m.anno = anno;

          BinaryMatch tmp = result;
          tmp.rhs = m;
          tmp.found = true;

          if(uniqueMatches.find(tmp) == uniqueMatches.end())
          {
            currentMatches.push_back(m);
            uniqueMatches.insert(tmp);
          }
        }
      }
    }

    if(currentMatches.empty())
    {
      // nothing found for this token, get the next one
      coveredTokenMatch = tokenCoveredByLHS->next();
    }

  } // end while

  if(!currentMatches.empty())
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
  if(left)
  {
    left->reset();
  }
  currentMatches.clear();
  if(tokenCoveredByLHS)
  {
    tokenCoveredByLHS->reset();
  }
  //lhsLeftTokenIt.reset();
  //tokenRightFromLHSIt.reset();
}

SeedOverlap::~SeedOverlap()
{
  delete tokenCoveredByLHS;
}

