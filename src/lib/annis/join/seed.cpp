#include <annis/join/seed.h>
#include <annis/annosearch/annotationsearch.h>

using namespace annis;

AnnoKeySeedJoin::AnnoKeySeedJoin(const DB &db, std::shared_ptr<Operator> op, 
                    std::shared_ptr<Iterator> lhs, size_t lhsIdx,
                   const std::set<AnnotationKey> &rightAnnoKeys)
  : db(db), op(op),
    left(lhs), lhsIdx(lhsIdx), rightAnnoKeys(rightAnnoKeys),
    currentMatchValid(false)
{

}

bool AnnoKeySeedJoin::next(std::vector<Match>& tuple)
{
  tuple.clear();

  if(!currentMatchValid)
  {
    nextLeftMatch();
  }
  
  if(!op || !left || !currentMatchValid || rightAnnoKeys.empty())
  {
    return false;
  }
  
  if(nextRightAnnotation())
  {
    tuple.reserve(currentLHSMatch.size()+1);
    tuple.insert(tuple.end(), currentLHSMatch.begin(), currentLHSMatch.end());
    tuple.push_back(currentRHSMatch);
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
        std::vector<Annotation> foundAnno =
            db.nodeAnnos.getAnnotations(currentRHSMatch.node, key.ns, key.name);
        if(!foundAnno.empty() && checkReflexitivity(currentLHSMatch[lhsIdx].node, currentLHSMatch[lhsIdx].anno, currentRHSMatch.node, foundAnno[0]))
        {
          currentRHSMatch.anno = foundAnno[0];
          
          tuple.reserve(currentLHSMatch.size()+1);
          tuple.insert(tuple.end(), currentLHSMatch.begin(), currentLHSMatch.end());
          tuple.push_back(currentRHSMatch);
          
          return true;
        }
      }
      else
      {
        // use the annotation keys as filter
        for(const auto& key : rightAnnoKeys)
        {
          std::vector<Annotation> foundAnno =
              db.nodeAnnos.getAnnotations(currentRHSMatch.node, key.ns, key.name);
          if(!foundAnno.empty())
          {
            matchingRightAnnos.push_back(foundAnno[0]);
          }
        }

        if(nextRightAnnotation())
        {
          tuple.reserve(currentLHSMatch.size()+1);
          tuple.insert(tuple.end(), currentLHSMatch.begin(), currentLHSMatch.end());
          tuple.push_back(currentRHSMatch);
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
}

bool AnnoKeySeedJoin::nextLeftMatch()
{
  matchingRightAnnos.clear();
  if(op && op->valid() && left && left->next(currentLHSMatch))
  {
    currentMatchValid = true;

    matchesByOperator = op->retrieveMatches(currentLHSMatch[lhsIdx]);
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
    if(checkReflexitivity(currentLHSMatch[lhsIdx].node, currentLHSMatch[lhsIdx].anno, currentRHSMatch.node, matchingRightAnnos.front()))
    {
      currentRHSMatch.anno = matchingRightAnnos.front();
      matchingRightAnnos.pop_front();
      
      return true;
    }
  }
  return false;
}

MaterializedSeedJoin::MaterializedSeedJoin(const DB &db, std::shared_ptr<Operator> op, 
                    std::shared_ptr<Iterator> lhs, size_t lhsIdx,
                   const std::unordered_set<Annotation>& rightAnno)
  : db(db), op(op),
    left(lhs), lhsIdx(lhsIdx), right(rightAnno),
    currentMatchValid(false)
{
}

bool MaterializedSeedJoin::next(std::vector<Match>& tuple)
{
  tuple.resize(0);
  
  if(!currentMatchValid)
  {
    nextLeftMatch();
  }
  
  // check some conditions where we can't perform a join
  if(!op || !left || !currentMatchValid || right.empty())
  {
    return false;
  }

  if(nextRightAnnotation())
  {
    tuple.reserve(currentLHSMatch.size()+1);
    tuple.insert(tuple.end(), currentLHSMatch.begin(), currentLHSMatch.end());
    tuple.push_back(currentRHSMatch);
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
            db.nodeAnnos.getAnnotations(currentRHSMatch.node, rightAnno.ns, rightAnno.name);
        if(!foundAnno.empty() && foundAnno[0].val == rightAnno.val
           && checkReflexitivity(currentLHSMatch[lhsIdx].node, currentLHSMatch[lhsIdx].anno, currentRHSMatch.node, foundAnno[0]))
        {
          currentRHSMatch.anno = foundAnno[0];
          
          tuple.reserve(currentLHSMatch.size()+1);
          tuple.insert(tuple.end(), currentLHSMatch.begin(), currentLHSMatch.end());
          tuple.push_back(currentRHSMatch);
          
          return true;
        }
      }
      else
      {
        // check all annotations which of them matches
        std::vector<Annotation> annos = db.nodeAnnos.getAnnotations(currentRHSMatch.node);
        for(const auto& a : annos)
        {
          if(right.find(a) != right.end())
          {
            matchingRightAnnos.push_back(a);
          }
        }

        if(nextRightAnnotation())
        {
          tuple.reserve(currentLHSMatch.size()+1);
          tuple.insert(tuple.end(), currentLHSMatch.begin(), currentLHSMatch.end());
          tuple.push_back(currentRHSMatch);
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
}

bool MaterializedSeedJoin::nextLeftMatch()
{  
  matchingRightAnnos.clear();
  if(op && op->valid() && left && left->next(currentLHSMatch))
  {
    currentMatchValid = true;

    matchesByOperator = op->retrieveMatches(currentLHSMatch[lhsIdx]);
    if(matchesByOperator)
    {
      return true;
    }
  }

  return false;
}

bool MaterializedSeedJoin::nextRightAnnotation()
{
  while(!matchingRightAnnos.empty())
  {
    if(checkReflexitivity(currentLHSMatch[lhsIdx].node, currentLHSMatch[lhsIdx].anno, currentRHSMatch.node, matchingRightAnnos.front()))
    {
      currentRHSMatch.anno = matchingRightAnnos.front();
      matchingRightAnnos.pop_front();
      return true;
    }
    else
    {
      matchingRightAnnos.pop_front();
    }
  }
  return false;
}

