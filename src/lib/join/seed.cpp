#include "seed.h"
#include "annotationsearch.h"

using namespace annis;

AnnoKeySeedJoin::AnnoKeySeedJoin(const DB &db, std::shared_ptr<Operator> op, std::shared_ptr<AnnoIt> lhs,
                   const std::set<AnnotationKey> &rightAnnoKeys)
  : db(db), op(op), currentMatchValid(false),
    left(lhs), rightAnnoKeys(rightAnnoKeys)
{
  nextLeftMatch();
}

BinaryMatch AnnoKeySeedJoin::next()
{
  currentMatch.found = false;

  if(!op || !left || !currentMatchValid || rightAnnoKeys.empty())
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

      if(rightAnnoKeys.size() == 1)
      {
        // only check the annotation key, not the value
        const AnnotationKey& key = *(rightAnnoKeys.begin());
        std::pair<bool, Annotation> foundAnno =
            db.getNodeAnnotation(currentMatch.rhs.node, key.ns, key.name);
        if(foundAnno.first && checkReflexitivity(currentMatch.lhs.node, currentMatch.lhs.anno, currentMatch.rhs.node, foundAnno.second))
        {
          currentMatch.found = true;
          currentMatch.rhs.anno = foundAnno.second;
          return currentMatch;
        }
      }
      else
      {
        // use the annotation keys as filter
        for(const auto& key : rightAnnoKeys)
        {
          std::pair<bool, Annotation> foundAnno =
              db.getNodeAnnotation(currentMatch.rhs.node, key.ns, key.name);
          if(foundAnno.first)
          {
            matchingRightAnnos.push_back(foundAnno.second);
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

void AnnoKeySeedJoin::reset()
{
  if(left)
  {
    left->reset();
  }

  matchesByOperator.reset(nullptr);
  matchingRightAnnos.clear();
  currentMatchValid = false;

  // start the iterations
  nextLeftMatch();

}

bool AnnoKeySeedJoin::nextLeftMatch()
{
  matchingRightAnnos.clear();
  if(op && op->valid() && left && left->hasNext())
  {
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

bool AnnoKeySeedJoin::nextRightAnnotation()
{
  while(!matchingRightAnnos.empty())
  {
    if(checkReflexitivity(currentMatch.lhs.node, currentMatch.lhs.anno, currentMatch.rhs.node, matchingRightAnnos.front()))
    {
      currentMatch.found = true;
      currentMatch.rhs.anno = matchingRightAnnos.front();
      matchingRightAnnos.pop_front();
      return true;
    }
  }
  return false;
}

MaterializedSeedJoin::MaterializedSeedJoin(const DB &db, std::shared_ptr<Operator> op, std::shared_ptr<AnnoIt> lhs,
                   const std::unordered_set<Annotation>& rightAnno)
  : db(db), op(op), currentMatchValid(false),
    left(lhs), right(rightAnno)
{
  nextLeftMatch();
}

BinaryMatch MaterializedSeedJoin::next()
{
  currentMatch.found = false;

  // check some conditions where we can't perform a join
  if(!op || !left || !currentMatchValid || right.empty())
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

      if(right.size() == 1)
      {
        // directly get the one node annotation
        const auto& rightAnno = *(right.begin());
        auto foundAnno =
            db.getNodeAnnotation(currentMatch.rhs.node, rightAnno.ns, rightAnno.name);
        if(foundAnno.first && foundAnno.second.val == rightAnno.val
           && checkReflexitivity(currentMatch.lhs.node, currentMatch.lhs.anno, currentMatch.rhs.node, foundAnno.second))
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

void MaterializedSeedJoin::reset()
{
  if(left)
  {
    left->reset();
  }

  matchesByOperator.reset(nullptr);
  matchingRightAnnos.clear();
  currentMatchValid = false;

  // start the iterations
  nextLeftMatch();

}

bool MaterializedSeedJoin::nextLeftMatch()
{  
  matchingRightAnnos.clear();
  if(op && op->valid() && left && left->hasNext())
  {

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

bool MaterializedSeedJoin::nextRightAnnotation()
{
  while(matchingRightAnnos.size() > 0)
  {
    if(checkReflexitivity(currentMatch.lhs.node, currentMatch.lhs.anno, currentMatch.rhs.node, matchingRightAnnos.front()))
    {
      currentMatch.found = true;
      currentMatch.rhs.anno = matchingRightAnnos.front();
      matchingRightAnnos.pop_front();
      return true;
    }
  }
  return false;
}

