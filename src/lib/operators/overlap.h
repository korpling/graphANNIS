#ifndef OVERLAP_H
#define OVERLAP_H

#include <set>
#include <list>

#include "../db.h"
#include "../annotationiterator.h"
#include "defaultjoins.h"
#include "../helper.h"

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

  TokenHelper tokenHelper;

};

class SeedOverlap : public Join
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
  LegacySeedJoin* tokenCoveredByLHS;
  //SeedJoin tokenRightFromLHSIt;
  std::list<Match> currentMatches;

  std::set<BinaryMatch, compBinaryMatch> uniqueMatches;

};
} // end namespace annis
#endif // OVERLAP_H
