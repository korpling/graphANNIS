#include "defaultjoins.h"
#include "annotationsearch.h"

using namespace annis;

NestedLoopJoin::NestedLoopJoin(const EdgeDB *edb, AnnotationIterator& left, AnnotationIterator& right, unsigned int minDistance, unsigned int maxDistance)
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
      if(edb->isConnected(initEdge(matchLeft.node, matchRight.node), minDistance, maxDistance))
      {
        result.found = true;
        result.lhs = matchLeft;
        result.rhs = matchRight;

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



SeedJoin::SeedJoin(const DB &db, const EdgeDB *edb, AnnotationIterator &left, Annotation right, unsigned int minDistance, unsigned int maxDistance)
  : db(db), edb(edb), left(left), right(right), minDistance(minDistance), maxDistance(maxDistance), edgeIterator(NULL)
{

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
    if(checkAnnotationEqual(candidateAnnotations[currentAnnotationCandidate], right))
    {
      result.found = true;
      result.lhs = matchLeft;
      result.rhs.node = connectedNode.second;
      result.rhs.anno = candidateAnnotations[currentAnnotationCandidate];
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


JoinWrapIterator::JoinWrapIterator(BinaryOperatorIterator &wrappedIterator, bool wrapLeftOperand)
  : matchAllAnnotation(initAnnotation()), wrappedIterator(wrappedIterator), wrapLeftOperand(wrapLeftOperand)
{
  currentMatch = wrappedIterator.next();
}

bool JoinWrapIterator::hasNext()
{
  return currentMatch.found;
}

Match JoinWrapIterator::next()
{
  Match result;
  if(currentMatch.found)
  {
    if(wrapLeftOperand)
    {
      result = currentMatch.lhs;
    }
    else
    {
      result = currentMatch.rhs;
    }
    currentMatch = wrappedIterator.next();
  }
  return result;
}

void JoinWrapIterator::reset()
{
  wrappedIterator.reset();
  currentMatch = wrappedIterator.next();
}



RightMostTokenForNodeIterator::RightMostTokenForNodeIterator(AnnotationIterator &source, const DB &db)
  : source(source), db(db), edb(db.getEdgeDB(ComponentType::RIGHT_TOKEN, annis_ns, ""))
{
  anyTokAnnotation = initAnnotation(db.getTokStringID(), 0, db.getNamespaceStringID());
}

bool RightMostTokenForNodeIterator::hasNext()
{
  return source.hasNext();
}

Match RightMostTokenForNodeIterator::next()
{
  Match result;
  if(source.hasNext() && edb != NULL)
  {
    currentOriginalMatch = source.next();

    // check if this is a token
    std::vector<Annotation> annos = db.getNodeAnnotationsByID(currentOriginalMatch.node);
    for(auto& a : annos)
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
  source.reset();
}

Match RightMostTokenForNodeIterator::currentNodeMatch()
{
  return currentOriginalMatch;
}


LeftMostTokenForNodeIterator::LeftMostTokenForNodeIterator(AnnotationIterator &source, const DB &db)
  : source(source), db(db), edb(db.getEdgeDB(ComponentType::LEFT_TOKEN, annis_ns, ""))
{
  anyTokAnnotation = initAnnotation(db.getTokStringID(), 0, db.getNamespaceStringID());
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
    std::vector<Annotation> annos = db.getNodeAnnotationsByID(currentOriginalMatch.node);
    for(auto& a : annos)
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


