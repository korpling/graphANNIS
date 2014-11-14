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
  NestedOverlap(DB &db, AnnotationIterator& left, AnnotationIterator& right);

  virtual BinaryMatch next();
  virtual void reset();

  virtual ~NestedOverlap();
private:
  AnnotationIterator& left;
  AnnotationIterator& right;


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
  SeedOverlap(DB &db, AnnotationIterator& left, AnnotationIterator& right);

  virtual BinaryMatch next();
  virtual void reset();

  virtual ~SeedOverlap();
private:

  AnnotationIterator& left;
  Annotation rightAnnotation;


  const DB& db;
  const EdgeDB* edbLeft;
  const EdgeDB* edbRight;
  const EdgeDB* edbOrder;
  const EdgeDB* edbCoverage;

  LeftMostTokenForNodeIterator lhsLeftTokenIt;
  /**
   * @brief finds *all* the token right from the lhs
   */
  SeedJoin tokenRightFromLHSIt;
  std::list<Match> currentMatches;

  std::set<BinaryMatch, compBinaryMatch> uniqueMatches;

};
} // end namespace annis
#endif // OVERLAP_H
