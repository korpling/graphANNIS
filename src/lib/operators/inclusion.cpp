#include "inclusion.h"

#include "../componenttypeiterator.h"

using namespace annis;

Inclusion::Inclusion(DB &db, AnnotationIterator &left, AnnotationIterator &right)
  : left(left), rightAnnotation(right.getAnnotation()), db(db), itCurrentCoveredToken(NULL)
{
  edbCoverage = db.getAllEdgeDBForType(ComponentType::COVERAGE);

  reset();
}

BinaryMatch Inclusion::next()
{
  BinaryMatch result;
  result.found = false;

  while(nextAnnotation())
  {

    result.right = currentRightMatch;
    result.left = currentLeftMatch;

    if(uniqueMatches.find(result) == uniqueMatches.end())
    {
      // not outputed yet
      uniqueMatches.insert(result);
      result.found = true;

      return result;
    }

  }

  return result;
}

void Inclusion::reset()
{
  uniqueMatches.clear();
  left.reset();

  currentAnnnotations.clear();
  itCurrentAnnotations = currentAnnnotations.begin();

  delete itCurrentCoveredToken;
}

Inclusion::~Inclusion()
{
  delete itCurrentCoveredToken;
}

bool Inclusion::nextAnnotation()
{
  do
  {
    while(itCurrentAnnotations != currentAnnnotations.end())
    {
      Annotation anno = *itCurrentAnnotations;
      itCurrentAnnotations++;

      if(checkAnnotationEqual(anno, rightAnnotation))
      {
        currentRightMatch.anno = anno;
        return true;
      }
    }
  } while(nextRightMatch());
  return false;
}

bool Inclusion::nextRightMatch()
{
  do
  {

    if(itRightMatchCandidates != rightMatchCandidates.end())
    {
      currentRightMatch.node = *itRightMatchCandidates;
      itRightMatchCandidates++;

      currentAnnnotations = db.getNodeAnnotationsByID(currentRightMatch.node);
      itCurrentAnnotations = currentAnnnotations.begin();

      return true;
    }

  } while(nextCoveredToken());

  return false;
}

bool Inclusion::nextCoveredToken()
{
  do
  {
    if(itCurrentCoveredToken != NULL)
    {
      for(std::pair<bool, nodeid_t> m = itCurrentCoveredToken->next(); m.first; m = itCurrentCoveredToken->next())
      {
        nodeid_t coveredToken = m.second;

        rightMatchCandidates.clear();
        for(const EdgeDB* edb : edbCoverage)
        {
          for(auto i : edb->getIncomingEdges(coveredToken))
          {
            rightMatchCandidates.push_back(i);
          }
        }
        itRightMatchCandidates = rightMatchCandidates.begin();

        return true;
      }
    }
  } while(nextLeftMatch());

  return false;
}



bool Inclusion::nextLeftMatch()
{
  while(left.hasNext())
  {
    currentLeftMatch = left.next();

    delete itCurrentCoveredToken;
    itCurrentCoveredToken = new ComponentTypeIterator(db, ComponentType::COVERAGE, currentLeftMatch.node);

    return true;
  }
  return false;
}
