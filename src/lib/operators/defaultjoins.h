#ifndef DEFAULTJOINS_H
#define DEFAULTJOINS_H

#include "types.h"
#include "annotationiterator.h"
#include "edgedb.h"
#include "db.h"

namespace annis
{

/** A join that checks all combinations of the left and right matches if their are connected. */
class NestedLoopJoin : public BinaryOperatorIterator
{
public:
  NestedLoopJoin(const EdgeDB* edb, AnnotationIterator &left, AnnotationIterator &right,
                 unsigned int minDistance = 1, unsigned int maxDistance = 1);
  virtual ~NestedLoopJoin();

  virtual BinaryMatch next();
  virtual void reset();
private:
  const EdgeDB* edb;
  AnnotationIterator& left;
  AnnotationIterator& right;
  unsigned int minDistance;
  unsigned int maxDistance;
  bool initialized;

  Match matchLeft;
  Match matchRight;

};

/** A join that takes the left argument as a seed, finds all connected nodes (matching the distance) and checks the condition for each node. */
class SeedJoin : public BinaryOperatorIterator
{
public:
  SeedJoin(const DB& db, const EdgeDB* edb, AnnotationIterator &left, Annotation right,
                 unsigned int minDistance = 1, unsigned int maxDistance = 1);
  virtual ~SeedJoin();

  virtual BinaryMatch next();
  virtual void reset();
private:
  const DB& db;
  const EdgeDB* edb;
  AnnotationIterator& left;
  Annotation right;
  unsigned int minDistance;
  unsigned int maxDistance;

  Match matchLeft;

  EdgeIterator* edgeIterator;
  std::pair<bool, nodeid_t> connectedNode;
  std::vector<Annotation> candidateAnnotations;
  size_t currentAnnotationCandidate;

  bool nextLeft();
  bool nextConnected();
  bool nextAnnotation();

};

class JoinWrapIterator : public AnnotationIterator
{
public:

  JoinWrapIterator(BinaryOperatorIterator& wrappedIterator, bool wrapLeftOperand = false);

  virtual bool hasNext();
  virtual Match next();
  virtual void reset();

  // TODO: is there any good way of defining this?
  virtual const Annotation& getAnnotation() {return matchAllAnnotation;}

  virtual ~JoinWrapIterator() {}
private:
  Annotation matchAllAnnotation;
  BinaryOperatorIterator& wrappedIterator;
  BinaryMatch currentMatch;
  bool wrapLeftOperand;
};

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

/**
 * @brief The LeftMostTokenForNodeIterator class
 *
 * This iterator outputs the token that is left aligned with the original matched node.
 * If the matched node itself is a token, the token is returned.
 */
class LeftMostTokenForNodeIterator : public AnnotationIterator
{
public:

  LeftMostTokenForNodeIterator(AnnotationIterator& source, const DB& db);

  virtual bool hasNext();
  virtual Match next();
  virtual void reset();

  virtual Match currentNodeMatch();

  virtual const Annotation& getAnnotation() {return source.getAnnotation();}

  virtual ~LeftMostTokenForNodeIterator() {}


private:
  AnnotationIterator& source;
  const DB& db;
  const EdgeDB* edb;
  Match matchTemplate;
  Match currentOriginalMatch;
  Annotation anyTokAnnotation;

  void initEdgeDB();
};

} // end namespace annis

#endif // DEFAULTJOINS_H
