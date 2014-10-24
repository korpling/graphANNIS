#include "defaultjoins.h"

using namespace annis;

NestedLoopJoin::NestedLoopJoin(const EdgeDB *edb, AnnotationIterator& left, AnnotationIterator& right, unsigned int minDistance, unsigned int maxDistance)
  : edb(edb), left(left), right(right), minDistance(minDistance), maxDistance(maxDistance), initialized(false)
{
}


BinaryMatch NestedLoopJoin::next()
{
  BinaryMatch result;
  result.found = false;

  bool proceed = true;

  if(!initialized)
  {
    proceed = false;
    if(left.hasNext())
    {
      matchLeft = left.next();
      proceed = true;
      initialized = true;
    }
  }

  while(proceed)
  {

    while(right.hasNext())
    {
      matchRight = right.next();

      // check the actual constraint
      if(edb->isConnected(initEdge(matchLeft.first, matchRight.first), minDistance, maxDistance))
      {
        result.found = true;
        result.left = matchLeft;
        result.right = matchRight;

        // immediatly return
        return result;
      }
    }
    if(left.hasNext())
    {
      matchLeft = left.next();
      right.reset();
    }
    else
    {
      proceed = false;
    }
  }
  return result;
}

void NestedLoopJoin::reset()
{
  left.reset();
  right.reset();
  initialized = false;
}

NestedLoopJoin::~NestedLoopJoin()
{

}



SeedJoin::SeedJoin(const DB &db, const EdgeDB *edb, AnnotationIterator &left, const Annotation &right, unsigned int minDistance, unsigned int maxDistance)
  : db(db), edb(edb), left(left), right(right), minDistance(minDistance), maxDistance(maxDistance), edgeIterator(NULL)
{

}

BinaryMatch SeedJoin::next()
{
  BinaryMatch result;
  result.found = false;

  if(nextAnnotation())
  {
    if(checkAnnotationEqual(candidateAnnotations[currentAnnotationCandidate], right))
    {
      result.found = true;
      result.left = matchLeft;
      result.right.first = connectedNode.second;
      result.right.second = candidateAnnotations[currentAnnotationCandidate];
      return result;
    }
  }

  return result;
}

void SeedJoin::reset()
{
  delete edgeIterator;
  edgeIterator = NULL;

  left.reset();

  currentAnnotationCandidate = 0;
  candidateAnnotations.clear();

  connectedNode.first = false;

}

bool SeedJoin::nextLeft()
{
  if(left.hasNext())
  {
    matchLeft = left.next();
    return true;
  }
  else
  {
    return false;
  }
}

bool SeedJoin::nextConnected()
{
  if(edgeIterator != NULL)
  {
    connectedNode = edgeIterator->next();
  }
  else
  {
    connectedNode.first = false;
  }

  if(!connectedNode.first)
  {
    delete edgeIterator;
    edgeIterator = NULL;
    if(nextLeft())
    {
      edgeIterator = edb->findConnected(matchLeft.first, minDistance, maxDistance);
      connectedNode = edgeIterator->next();
    }
    else
    {
      return false;
    }
  }

  return connectedNode.first;
}

bool SeedJoin::nextAnnotation()
{
  currentAnnotationCandidate++;
  if(currentAnnotationCandidate >= candidateAnnotations.size())
  {
    currentAnnotationCandidate = 0;
    if(nextConnected())
    {
      candidateAnnotations = db.getNodeAnnotationsByID(connectedNode.second);
    }
    else
    {
      return false;
    }
  }
  return currentAnnotationCandidate < candidateAnnotations.size();
}

SeedJoin::~SeedJoin()
{
  delete edgeIterator;
}
