#ifndef OVERLAP_H
#define OVERLAP_H

#include <set>
#include <list>

#include "../db.h"
#include "../annotationiterator.h"
#include "defaultjoins.h"

namespace annis
{

class Overlap : public BinaryOperatorIterator
{
public:
  Overlap(DB &db, AnnotationIterator& left, AnnotationIterator& right);

  virtual BinaryMatch next();
  virtual void reset();

  virtual ~Overlap();
private:

  AnnotationIterator& left;
  Annotation rightAnnotation;


  const DB& db;
  const EdgeDB* edbLeft;
  const EdgeDB* edbRight;
  const EdgeDB* edbOrder;

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
