#include "precedence.h"
#include "defaultjoins.h"

using namespace annis;

Precedence::Precedence(DB &db, AnnotationIterator& left, AnnotationIterator& right,
                       unsigned int minDistance, unsigned int maxDistance)
  : left(left), right(right), minDistance(minDistance), maxDistance(maxDistance), actualIterator(NULL)
{
  const EdgeDB* edbOrder = db.getEdgeDB(ComponentType::ORDERING, annis_ns, "");
  const EdgeDB* edbLeft = db.getEdgeDB(ComponentType::LEFT_TOKEN, annis_ns, "");
  const EdgeDB* edbRight = db.getEdgeDB(ComponentType::RIGHT_TOKEN, annis_ns, "");

  if(edbOrder != NULL && edbLeft != NULL && edbRight != NULL)
  {
    // TODO: allow to use a nested loop iterator instead
    actualIterator = new SeedJoin(db, edbOrder, left, right.getAnnotation(), minDistance, maxDistance);
  }
}

Precedence::~Precedence()
{
  delete actualIterator;
}

BinaryMatch Precedence::next()
{

}

void Precedence::reset()
{

}
