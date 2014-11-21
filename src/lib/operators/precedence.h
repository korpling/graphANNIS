#ifndef PRECEDENCE_H
#define PRECEDENCE_H

#include "db.h"
#include "../annotationiterator.h"
#include "defaultjoins.h"

#include <list>
#include <stack>

namespace annis
{


class Precedence : public BinaryOperatorIterator
{
public:
  Precedence(DB &db, AnnotationIterator& left, AnnotationIterator& right,
             unsigned int minDistance=1, unsigned int maxDistance=1);
  virtual ~Precedence();

  virtual BinaryMatch next();
  virtual void reset();

private:
  const DB& db;
  AnnotationIterator& left;
  AnnotationIterator& right;
  unsigned int minDistance;
  unsigned int maxDistance;

  RightMostTokenForNodeIterator tokIteratorForLeftNode;
  const Annotation& annoForRightNode;

  BinaryOperatorIterator* actualJoin;
  std::stack<Match, std::list<Match>> currentMatches;
  BinaryMatch currentMatchedToken;

  const EdgeDB* edbLeft;
};





} // end namespace annis

#endif // PRECEDENCE_H
