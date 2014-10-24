#include "defaultjoins.h"

using namespace annis;

NestedLoopJoin::NestedLoopJoin(const EdgeDB *edb, AnnotationIterator& left, AnnotationIterator& right, unsigned int minDistance, unsigned int maxDistance)
  : edb(edb), left(left), right(right), minDistance(minDistance), maxDistance(maxDistance)
{
}


BinaryMatch NestedLoopJoin::next()
{
  BinaryMatch result;
  result.found = false;


  while(left.hasNext())
  {
    matchLeft = left.next();
    right.reset();

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
  }
  return result;
}

void NestedLoopJoin::reset()
{
  left.reset();
  right.reset();
}

NestedLoopJoin::~NestedLoopJoin()
{

}



SeedJoin::SeedJoin(const DB &db, const EdgeDB *edb, AnnotationIterator &left, const Annotation &right, unsigned int minDistance, unsigned int maxDistance)
  : db(db), edb(edb), left(left), right(right), minDistance(minDistance), maxDistance(maxDistance)
{

}

BinaryMatch SeedJoin::next()
{
  BinaryMatch result;
  result.found = false;

  while(left.hasNext())
  {
    Match matchLeft = left.next();
    EdgeIterator* connectedEdges = edb->findConnected(matchLeft.first, minDistance, maxDistance);
    for(std::pair<bool, nodeid_t> candidateRight = connectedEdges->next(); candidateRight.first; candidateRight = connectedEdges->next())
    {
      // check all annnotations
      std::vector<Annotation> candidateAnnotations = db.getNodeAnnotationsByID(candidateRight.second);
      for(size_t i=0; i < candidateAnnotations.size(); i++)
      {
        if(checkAnnotationEqual(candidateAnnotations[i], right))
        {
          result.found = true;
          result.left = matchLeft;
          result.right = Match(candidateRight.second, candidateAnnotations[i]);
          delete connectedEdges;
          return result;
        }
      }

    }
    delete connectedEdges;
  }

  return result;
}

void SeedJoin::reset()
{
  left.reset();
}


SeedJoin::~SeedJoin()
{

}
