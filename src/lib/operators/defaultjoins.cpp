#include "defaultjoins.h"
#include "annotationsearch.h"

using namespace annis;

NestedLoopJoin::NestedLoopJoin(const EdgeDB *edb, std::shared_ptr<AnnoIt> left, std::shared_ptr<AnnoIt> right, unsigned int minDistance, unsigned int maxDistance)
  : edb(edb), left(left), right(right), minDistance(minDistance), maxDistance(maxDistance), initialized(false)
{
}

BinaryMatch NestedLoopJoin::next()
{
  BinaryMatch result;
  result.found = false;

  if(edb == NULL)
  {
    return result;
  }

  bool proceed = true;

  if(!initialized)
  {
    proceed = false;
    if(left->hasNext())
    {
      matchLeft = left->next();
      proceed = true;
      initialized = true;
    }
  }

  while(proceed)
  {

    while(right->hasNext())
    {
      matchRight = right->next();

      // check the actual constraint
      if(edb->isConnected(Init::initEdge(matchLeft.node, matchRight.node), minDistance, maxDistance))
      {
        result.found = true;
        result.lhs = matchLeft;
        result.rhs = matchRight;

        // immediatly return
        return result;
      }
    }
    if(left->hasNext())
    {
      matchLeft = left->next();
      right->reset();
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
  left->reset();
  right->reset();
  initialized = false;
}

NestedLoopJoin::~NestedLoopJoin()
{

}

void NestedLoopJoin::init(std::shared_ptr<AnnoIt> lhs, std::shared_ptr<AnnoIt> rhs)
{
  left = lhs;
  right = rhs;
}



SeedJoin::SeedJoin(const DB &db, const EdgeDB *edb, std::shared_ptr<AnnoIt> left, Annotation right, unsigned int minDistance, unsigned int maxDistance)
  : db(db), edb(edb), left(left), right(right), minDistance(minDistance), maxDistance(maxDistance), edgeIterator(NULL), anyNodeShortcut(false)
{
  if(right.name == db.getNodeNameStringID() && right.ns == db.getNamespaceStringID() && right.val == 0)
  {
    anyNodeShortcut = true;
  }
  reset();
}

BinaryMatch SeedJoin::next()
{
  BinaryMatch result;
  result.found = false;

  if(edb == NULL)
  {
    return result;
  }


  while(nextAnnotation())
  {
    if(anyNodeShortcut)
    {
      result.found = true;
      result.lhs = matchLeft;
      result.rhs.node = connectedNode.second;
      result.rhs.anno = right;
      return result;
    }
    else if(checkAnnotationEqual(*currentAnnotationCandidate, right))
    {
      result.found = true;
      result.lhs = matchLeft;
      result.rhs.node = connectedNode.second;
      result.rhs.anno = *currentAnnotationCandidate;
      return result;
    }
  }


  return result;
}

void SeedJoin::reset()
{
  delete edgeIterator;
  edgeIterator = NULL;

  left->reset();

  candidateAnnotations.clear();
  currentAnnotationCandidate = candidateAnnotations.begin();

  connectedNode.first = false;

}

bool SeedJoin::nextLeft()
{
  if(left->hasNext())
  {
    matchLeft = left->next();
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

  while(!connectedNode.first)
  {
    delete edgeIterator;
    edgeIterator = NULL;
    if(nextLeft())
    {
      edgeIterator = edb->findConnected(matchLeft.node, minDistance, maxDistance);
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
  if(anyNodeShortcut)
  {
    return nextConnected();
  }
  else
  {
    currentAnnotationCandidate++;
    if(currentAnnotationCandidate == candidateAnnotations.end())
    {
      if(nextConnected())
      {
        candidateAnnotations = db.getNodeAnnotationsByID(connectedNode.second);
        currentAnnotationCandidate = candidateAnnotations.begin();
      }
      else
      {
        return false;
      }
    }
    return currentAnnotationCandidate != candidateAnnotations.end();
  }
}

SeedJoin::~SeedJoin()
{
  delete edgeIterator;
}

void SeedJoin::init(std::shared_ptr<AnnoIt> lhs, std::shared_ptr<AnnoIt> rhs)
{
  left = lhs;
  right = rhs->getAnnotation();
}


RightMostTokenForNodeIterator::RightMostTokenForNodeIterator(std::shared_ptr<AnnoIt> source, const DB &db)
  : source(source), db(db), edb(db.getEdgeDB(ComponentType::RIGHT_TOKEN, annis_ns, "")), tokenShortcut(false)
{
  anyTokAnnotation = Init::initAnnotation(db.getTokStringID(), 0, db.getNamespaceStringID());
  const Annotation& anno = source->getAnnotation();
  if(anno.name == db.getTokStringID() && anno.ns == db.getNamespaceStringID() && anno.val == 0)
  {
    tokenShortcut = true;
  }
}

bool RightMostTokenForNodeIterator::hasNext()
{
  return source->hasNext();
}

Match RightMostTokenForNodeIterator::next()
{
  Match result;
  if(source->hasNext() && edb != NULL)
  {
    currentOriginalMatch = source->next();

    // check if we can use the shortcut
    if(tokenShortcut)
    {
      return currentOriginalMatch;
    }

    // we still have to check if this is a token in case for token annotions
    for(const auto& a : db.getNodeAnnotationsByID(currentOriginalMatch.node))
    {
      if(checkAnnotationEqual(anyTokAnnotation, a))
      {
        return currentOriginalMatch;
      }
    }

    result.node = edb->getOutgoingEdges(currentOriginalMatch.node)[0];
    result.anno.name = db.getTokStringID();
    result.anno.ns = db.getNamespaceStringID();
    result.anno.val = 0; //TODO: do we want to include the actual value here?
  }

  return result;
}

void RightMostTokenForNodeIterator::reset()
{
  source->reset();
}

const Match& RightMostTokenForNodeIterator::currentNodeMatch()
{
  return currentOriginalMatch;
}


LeftMostTokenForNodeIterator::LeftMostTokenForNodeIterator(AnnoIt &source, const DB &db)
  : source(source), db(db), edb(db.getEdgeDB(ComponentType::LEFT_TOKEN, annis_ns, ""))
{
  anyTokAnnotation = Init::initAnnotation(db.getTokStringID(), 0, db.getNamespaceStringID());
}

bool LeftMostTokenForNodeIterator::hasNext()
{
  return source.hasNext();
}

Match LeftMostTokenForNodeIterator::next()
{
  Match result;
  if(source.hasNext() && edb != NULL)
  {
    currentOriginalMatch = source.next();

    // check if this is a token
    for(const auto& a : db.getNodeAnnotationsByID(currentOriginalMatch.node))
    {
      if(checkAnnotationEqual(anyTokAnnotation, a))
      {
        return currentOriginalMatch;
      }
    }

    result.node = edb->getOutgoingEdges(currentOriginalMatch.node)[0];
    result.anno.name = db.getTokStringID();
    result.anno.ns = db.getNamespaceStringID();
    result.anno.val = 0; //TODO: do we want to include the actual value here?
  }

  return result;
}

void LeftMostTokenForNodeIterator::reset()
{
  source.reset();
}

Match LeftMostTokenForNodeIterator::currentNodeMatch()
{
  return currentOriginalMatch;
}


