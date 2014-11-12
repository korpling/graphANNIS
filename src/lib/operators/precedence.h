#ifndef PRECEDENCE_H
#define PRECEDENCE_H

#include "db.h"
#include "../annotationiterator.h"

namespace annis
{

/**
 * @brief The RightMostTokenForNodeIterator class
 *
 * This iterator outputs the token that is right aligned with the original matched node.
 * If the matched node itself is a token, the token is returned.
 */
class RightMostTokenForNodeIterator : public AnnotationIterator
{
public:

  RightMostTokenForNodeIterator(AnnotationIterator& source, const DB& db);

  virtual bool hasNext();
  virtual Match next();
  virtual void reset();

  virtual Match currentNodeMatch();

  virtual const Annotation& getAnnotation() {return source.getAnnotation();}

  virtual ~RightMostTokenForNodeIterator() {}


private:
  AnnotationIterator& source;
  const DB& db;
  const EdgeDB* edb;
  Match matchTemplate;
  Match currentOriginalMatch;
  Annotation anyTokAnnotation;

  void initEdgeDB();
};



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
  std::vector<nodeid_t> currentNodeCandiates;
  BinaryMatch currentMatchedToken;

  const EdgeDB* edbLeft;
};





} // end namespace annis

#endif // PRECEDENCE_H
