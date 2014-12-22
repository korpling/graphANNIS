#ifndef OVERLAP_H
#define OVERLAP_H

#include <set>
#include <list>

#include "../db.h"
#include "../annotationiterator.h"
#include "defaultjoins.h"

namespace annis
{

class NestedOverlap : public BinaryIt
{
public:
  NestedOverlap(DB &db, std::shared_ptr<AnnoIt> left, std::shared_ptr<AnnoIt> right);

  virtual void init(std::shared_ptr<AnnoIt> lhs, std::shared_ptr<AnnoIt> rhs);

  virtual BinaryMatch next();
  virtual void reset();

  virtual ~NestedOverlap();
private:
  std::shared_ptr<AnnoIt> left;
  std::shared_ptr<AnnoIt> right;


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

class SeedOverlap : public BinaryIt
{
public:
  SeedOverlap(DB &db);

  virtual void init(std::shared_ptr<AnnoIt> lhs, std::shared_ptr<AnnoIt> rhs);

  virtual BinaryMatch next();
  virtual void reset();

  virtual ~SeedOverlap();
private:


  const DB& db;

  std::shared_ptr<AnnoIt> left;
  Annotation rightAnnotation;
  Annotation anyNodeAnno;


  const EdgeDB* edbLeft;
  const EdgeDB* edbRight;
  const EdgeDB* edbOrder;
  const EdgeDB* edbCoverage;

  //LeftMostTokenForNodeIterator lhsLeftTokenIt;
  SeedJoin* tokenCoveredByLHS;
  //SeedJoin tokenRightFromLHSIt;
  std::list<Match> currentMatches;

  std::set<BinaryMatch, compBinaryMatch> uniqueMatches;

};
} // end namespace annis
#endif // OVERLAP_H
