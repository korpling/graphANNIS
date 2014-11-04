#include "overlap.h"

using namespace annis;

Overlap::Overlap(DB &db, AnnotationIterator &left, AnnotationIterator &right)
  : left(left), rightAnnotation(right.getAnnotation()), db(db), edbLeft(NULL), edbRight(NULL),
    leftTokBorder(0), rightTokBorder(0), currentTok(0)
{
  edbLeft = db.getEdgeDB(ComponentType::LEFT_TOKEN, annis_ns, "");
  edbRight = db.getEdgeDB(ComponentType::RIGHT_TOKEN, annis_ns, "");

  reset();
}

BinaryMatch Overlap::next()
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

void Overlap::reset()
{
  uniqueMatches.clear();
  left.reset();
  currentAnnnotations.clear();
  itCurrentAnnotations = currentAnnnotations.begin();
  nodesOverlappingCurrentToken.clear();
  itNodeOverlappingCurrentToken = nodesOverlappingCurrentToken.begin();

  leftTokBorder = 0;
  rightTokBorder = 0;
  currentTok = 1;
}

Overlap::~Overlap()
{

}

bool Overlap::nextAnnotation()
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
  } while(nextOverlappingNode());
  return false;
}

bool Overlap::nextOverlappingNode()
{
  do
  {
    while(itNodeOverlappingCurrentToken != nodesOverlappingCurrentToken.end())
    {
      nodeid_t currentNode = *itNodeOverlappingCurrentToken;

      currentRightMatch.node = currentNode;

      currentAnnnotations = db.getNodeAnnotationsByID(currentNode);
      itCurrentAnnotations = currentAnnnotations.begin();
      itNodeOverlappingCurrentToken++;

      return true;
    }
  } while(nextToken());

  return false;
}

bool Overlap::nextToken()
{
  do
  {
    while(currentTok <= rightTokBorder)
    {
      // get all the nodes that cover this token

      nodesOverlappingCurrentToken.clear();
      nodesOverlappingCurrentToken.insert(currentTok);
      for(auto n : edbLeft->getOutgoingEdges(currentTok))
      {
        nodesOverlappingCurrentToken.insert(n);
      }
      for(auto n : edbRight->getOutgoingEdges(currentTok))
      {
        nodesOverlappingCurrentToken.insert(n);
      }
      itNodeOverlappingCurrentToken = nodesOverlappingCurrentToken.begin();
      currentTok++;

      return true;
    }
  } while(nextMatch());

  return false;
}

bool Overlap::nextMatch()
{
  while(left.hasNext())
  {
    currentLeftMatch = left.next();
    // get the covered token for the matched node
    leftTokBorder = edbLeft->getOutgoingEdges(currentLeftMatch.node)[0];
    rightTokBorder = edbRight->getOutgoingEdges(currentLeftMatch.node)[0];
    currentTok = leftTokBorder;
    return true;
  }
  return false;
}
