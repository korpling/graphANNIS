#include "seed.h"
#include "annotationsearch.h"

using namespace annis;

SeedJoin::SeedJoin(const DB &db, std::shared_ptr<Operator> op, std::shared_ptr<AnnoIt> lhs, const std::unordered_set<Annotation>& rightAnno)
  : db(db), op(op), currentMatchValid(false), anyNodeShortcut(false),
    left(lhs), right(rightAnno)
{
  anyNodeShortcut = false;
  Annotation anyNodeAnno = Init::initAnnotation(db.getNodeNameStringID(), 0, db.getNamespaceStringID());

  if(right.size() == 1)
  {
    Annotation anno = *(right.begin());
    if(checkAnnotationEqual(anno, anyNodeAnno))
    {
      anyNodeShortcut = true;
    }
  }

  nextLeftMatch();
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
      else if(right.size() == 1)
      {
        // directly get the one node annotation
        const Annotation& rightAnno = *(right.begin());
        std::pair<bool, Annotation> foundAnno =
            db.getNodeAnnotation(currentMatch.rhs.node, rightAnno.ns, rightAnno.name);
        if(foundAnno.first && foundAnno.second.val == rightAnno.val)
        {
          currentMatch.found = true;
          currentMatch.rhs.anno = foundAnno.second;
          return currentMatch;
        }
      }
      else
      {
        // check all annotations which of them matches
        std::list<Annotation> annos = db.getNodeAnnotationsByID(currentMatch.rhs.node);
        for(const auto& a : annos)
        {
          if(right.find(a) != right.end())
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

void SeedJoin::reset()
{
  if(left)
  {
    left->reset();
  }

  matchesByOperator.release();
  matchingRightAnnos.clear();
  currentMatchValid = false;

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

