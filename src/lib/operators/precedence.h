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
  Precedence(DB &db, std::shared_ptr<AnnotationIterator> left, std::shared_ptr<AnnotationIterator> right,
             unsigned int minDistance=1, unsigned int maxDistance=1);
  virtual ~Precedence();

  virtual BinaryMatch next();
  virtual void reset();

private:
  const DB& db;
  std::shared_ptr<AnnotationIterator> left;
  std::shared_ptr<AnnotationIterator> right;
  unsigned int minDistance;
  unsigned int maxDistance;

  RightMostTokenForNodeIterator tokIteratorForLeftNode;
  const Annotation& annoForRightNode;

  BinaryOperatorIterator* actualJoin;
  std::stack<Match, std::list<Match>> currentMatches;
  BinaryMatch currentMatchedToken;

  const EdgeDB* edbLeft;
  bool tokenShortcut;
};





} // end namespace annis

#endif // PRECEDENCE_H
