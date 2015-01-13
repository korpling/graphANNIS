#include "seed.h"
#include "annotationsearch.h"

using namespace annis;

SeedJoin::SeedJoin(const DB &db, std::shared_ptr<Operator> op)
  : db(db), op(op), currentMatchValid(false), anyNodeShortcut(false)
{
  reset();
}

BinaryMatch SeedJoin::next()
{
  currentMatch.found = false;

  if(!op || !left || !currentMatchValid)
  {
    return currentMatch;
  }

  if(nextRightAnnotation())
  {
    return currentMatch;
  }

  do
  {
    while(matchesByOperator && matchesByOperator->hasNext())
    {
      currentMatch.rhs = matchesByOperator->next();

      if(anyNodeShortcut)
      {
        currentMatch.found = true;
        std::pair<bool, Annotation> annoSearch =
            db.getNodeAnnotation(currentMatch.rhs.node, db.getNamespaceStringID(),
                                 db.getNodeNameStringID());
        if(annoSearch.first)
        {
          currentMatch.rhs.anno = annoSearch.second;
        }
        return currentMatch;
      }
      else
      {
        // check all annotations which of them matches
        std::list<Annotation> annos = db.getNodeAnnotationsByID(currentMatch.rhs.node);
        for(const auto& a : annos)
        {
          if(checkAnnotationEqual(a, right))
          {
            matchingRightAnnos.push_back(a);
          }
        }
        if(nextRightAnnotation())
        {
          return currentMatch;
        }
      }
    } // end while there are right candidates
  } while(nextLeftMatch()); // end while left has match


  return currentMatch;
}


void SeedJoin::init(std::shared_ptr<AnnoIt> lhs, std::shared_ptr<AnnoIt> rhs)
{
  left = lhs;
  anyNodeShortcut = false;
  if(rhs)
  {
    Annotation anyNodeAnno = Init::initAnnotation(db.getNodeNameStringID(), 0, db.getNamespaceStringID());
    if(checkAnnotationEqual(rhs->getAnnotation(), anyNodeAnno))
    {
      anyNodeShortcut = true;
    }
    else
    {
      right = rhs->getAnnotation();
    }
  }
  else
  {
    anyNodeShortcut = true;
  }

  nextLeftMatch();
}


void SeedJoin::reset()
{
  if(left)
  {
    left->reset();
  }

  currentMatch.lhs.node = 6666666;

  matchesByOperator.release();
  matchingRightAnnos.clear();
  currentMatchValid = false;
  anyNodeShortcut = false;

  // start the iterations
  nextLeftMatch();

}

bool SeedJoin::nextLeftMatch()
{
  if(left && left->hasNext())
  {
    matchesByOperator.release();
    matchingRightAnnos.clear();

    currentMatch.lhs = left->next();
    currentMatchValid = true;

    matchesByOperator = op->retrieveMatches(currentMatch.lhs);
    if(!matchesByOperator)
    {
      std::cerr << "could not create right matches from operator!" << std::endl;
    }


    return true;
  }

  return false;
}

bool SeedJoin::nextRightAnnotation()
{
  if(matchingRightAnnos.size() > 0)
  {
    currentMatch.found = true;
    currentMatch.rhs.anno = matchingRightAnnos.front();
    matchingRightAnnos.pop_front();
    return true;
  }
  return false;
}

SeedJoin::~SeedJoin()
{
}

