#ifndef OVERLAP_H
#define OVERLAP_H

#include <set>
#include <list>

#include "../db.h"
#include "../annotationiterator.h"
#include "defaultjoins.h"

namespace annis
{

class NestedOverlap : public BinaryOperatorIterator
{
public:
  NestedOverlap(DB &db, std::shared_ptr<AnnotationIterator> left, std::shared_ptr<AnnotationIterator> right);

  virtual void init(std::shared_ptr<AnnotationIterator> lhs, std::shared_ptr<AnnotationIterator> rhs);

  virtual BinaryMatch next();
  virtual void reset();

  virtual ~NestedOverlap();
private:
  std::shared_ptr<AnnotationIterator> left;
  std::shared_ptr<AnnotationIterator> right;


  const DB& db;
  const EdgeDB* edbLeft;
  const EdgeDB* edbRight;
  const EdgeDB* edbOrder;

  bool initialized;

  Match matchLHS;
  Match matchRHS;

  nodeid_t leftTokenForNode(nodeid_t n);
  nodeid_t rightTokenForNode(nodeid_t n);
  bool isToken(nodeid_t n);

};

class SeedOverlap : public BinaryOperatorIterator
{
public:
  SeedOverlap(DB &db, std::shared_ptr<AnnotationIterator> left, std::shared_ptr<AnnotationIterator> right);

  virtual void init(std::shared_ptr<AnnotationIterator> lhs, std::shared_ptr<AnnotationIterator> rhs);

  virtual BinaryMatch next();
  virtual void reset();

  virtual ~SeedOverlap();
private:


  const DB& db;

  std::shared_ptr<AnnotationIterator> left;
  Annotation rightAnnotation;
  Annotation anyNodeAnno;


  const EdgeDB* edbLeft;
  const EdgeDB* edbRight;
  const EdgeDB* edbOrder;
  const EdgeDB* edbCoverage;

  //LeftMostTokenForNodeIterator lhsLeftTokenIt;
  SeedJoin tokenCoveredByLHS;
  //SeedJoin tokenRightFromLHSIt;
  std::list<Match> currentMatches;

  std::set<BinaryMatch, compBinaryMatch> uniqueMatches;

};
} // end namespace annis
#endif // OVERLAP_H
