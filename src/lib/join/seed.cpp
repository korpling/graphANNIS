#include <annis/join/seed.h>
#include <annis/annosearch/annotationsearch.h>

using namespace annis;

AnnoKeySeedJoin::AnnoKeySeedJoin(const DB &db, std::shared_ptr<Operator> op, std::shared_ptr<AnnoIt> lhs,
                   const std::set<AnnotationKey> &rightAnnoKeys)
  : db(db), op(op), currentMatchValid(false),
    left(lhs), rightAnnoKeys(rightAnnoKeys)
{
  nextLeftMatch();
}

bool AnnoKeySeedJoin::next(Match& lhsMatch, Match& rhsMatch)
{
  bool found = false;

  if(!op || !left || !currentMatchValid || rightAnnoKeys.empty())
  {
    return false;
  }

  if(nextRightAnnotation())
  {
    return true;
  }

  do
  {
    while(matchesByOperator && matchesByOperator->next(currentRHSMatch))
    {
      if(rightAnnoKeys.size() == 1)
      {
        // only check the annotation key, not the value
        const AnnotationKey& key = *(rightAnnoKeys.begin());
        std::pair<bool, Annotation> foundAnno =
            db.nodeAnnos.getNodeAnnotation(currentRHSMatch.node, key.ns, key.name);
        if(foundAnno.first && checkReflexitivity(currentLHSMatch.node, currentLHSMatch.anno, currentRHSMatch.node, foundAnno.second))
        {
          currentRHSMatch.anno = foundAnno.second;
          
          lhsMatch = currentLHSMatch;
          rhsMatch = currentRHSMatch;
          
          return true;
        }
      }
      else
      {
        // use the annotation keys as filter
        for(const auto& key : rightAnnoKeys)
        {
          std::pair<bool, Annotation> foundAnno =
              db.nodeAnnos.getNodeAnnotation(currentRHSMatch.node, key.ns, key.name);
          if(foundAnno.first)
          {
            matchingRightAnnos.push_back(foundAnno.second);
          }
        }

        if(nextRightAnnotation())
        {
          lhsMatch = currentLHSMatch;
          rhsMatch = currentRHSMatch;
          return true;
        }
      }
    } // end while there are right candidates
  } while(nextLeftMatch()); // end while left has match


  return false;
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
  if(op && op->valid() && left && left->next(currentLHSMatch))
  {
    currentMatchValid = true;

    matchesByOperator = op->retrieveMatches(currentLHSMatch);
    if(matchesByOperator)
    {
      return true;
    }
  }

  return false;
}

bool AnnoKeySeedJoin::nextRightAnnotation()
{
  while(!matchingRightAnnos.empty())
  {
    if(checkReflexitivity(currentLHSMatch.node, currentLHSMatch.anno, currentRHSMatch.node, matchingRightAnnos.front()))
    {
      currentRHSMatch.anno = matchingRightAnnos.front();
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

bool MaterializedSeedJoin::next(Match& lhsMatch, Match& rhsMatch)
{
  // check some conditions where we can't perform a join
  if(!op || !left || !currentMatchValid || right.empty())
  {
    return false;
  }

  if(nextRightAnnotation())
  {
    lhsMatch = currentLHSMatch;
    rhsMatch = currentRHSMatch;
    return true;
  }

  do
  {
    while(matchesByOperator && matchesByOperator->next(currentRHSMatch))
    {
      if(right.size() == 1)
      {
        // directly get the one node annotation
        const auto& rightAnno = *(right.begin());
        auto foundAnno =
            db.nodeAnnos.getNodeAnnotation(currentRHSMatch.node, rightAnno.ns, rightAnno.name);
        if(foundAnno.first && foundAnno.second.val == rightAnno.val
           && checkReflexitivity(currentLHSMatch.node, currentLHSMatch.anno, currentRHSMatch.node, foundAnno.second))
        {
          currentRHSMatch.anno = foundAnno.second;
          lhsMatch = currentLHSMatch;
          rhsMatch = currentRHSMatch;
          return true;
        }
      }
      else
      {
        // check all annotations which of them matches
        std::list<Annotation> annos = db.nodeAnnos.getNodeAnnotationsByID(currentRHSMatch.node);
        for(const auto& a : annos)
        {
          if(right.find(a) != right.end())
          {
            matchingRightAnnos.push_back(a);
          }
        }

        if(nextRightAnnotation())
        {
          lhsMatch = currentLHSMatch;
          rhsMatch = currentRHSMatch;
          return true;
        }
      }
    } // end while there are right candidates
  } while(nextLeftMatch()); // end while left has match


  return false;
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
  if(op && op->valid() && left && left->next(currentLHSMatch))
  {
    currentMatchValid = true;

    matchesByOperator = op->retrieveMatches(currentLHSMatch);
    if(matchesByOperator)
    {
      return true;
    }
  }

  return false;
}

bool MaterializedSeedJoin::nextRightAnnotation()
{
  while(matchingRightAnnos.size() > 0)
  {
    if(checkReflexitivity(currentLHSMatch.node, currentLHSMatch.anno, currentRHSMatch.node, matchingRightAnnos.front()))
    {
      currentRHSMatch.anno = matchingRightAnnos.front();
      matchingRightAnnos.pop_front();
      return true;
    }
  }
  return false;
}

