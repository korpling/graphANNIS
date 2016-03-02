#pragma once

#include <annis/types.h>
#include <annis/iterators.h>
#include <annis/operators/operator.h>
#include <annis/graphstorage/graphstorage.h>
#include <annis/db.h>
#include <annis/util/comparefunctions.h>

#include <unordered_set>

namespace annis
{

/** A join that takes the left argument as a seed, finds all connected nodes (matching the distance) and checks the condition for each node. */
class AnnoKeySeedJoin : public Iterator
{
public:
  AnnoKeySeedJoin(const DB& db, std::shared_ptr<Operator> op,
           std::shared_ptr<Iterator> lhs,
            size_t lhsIdx,
           const std::set<AnnotationKey> &rightAnnoKeys);
  virtual ~AnnoKeySeedJoin() {}

  virtual bool next(std::vector<Match>& tuple) override;
  virtual void reset() override;
private:
  const DB& db;
  std::shared_ptr<Operator> op;

  std::shared_ptr<Iterator> left;
  size_t lhsIdx;
  const std::set<AnnotationKey>& rightAnnoKeys;
  unsigned int minDistance;
  unsigned int maxDistance;

  std::unique_ptr<AnnoIt> matchesByOperator;
  std::vector<Match> currentLHSMatch;
  Match currentRHSMatch;
  bool currentMatchValid;
  std::list<Annotation> matchingRightAnnos;

  bool nextLeftMatch();
  bool nextRightAnnotation();

  bool checkReflexitivity(const nodeid_t& lhsNode, const Annotation& lhsAnno, const nodeid_t& rhsNode, const Annotation& rhsAnno)
  {
    if(!op->isReflexive() && lhsNode == rhsNode && checkAnnotationKeyEqual(lhsAnno, rhsAnno))
    {
      return false;
    }
    else
    {
      return true;
    }
  }

};

/**
 * @brief The MaterializedSeedJoin class
 */
class MaterializedSeedJoin : public Iterator
{
public:
  MaterializedSeedJoin(const DB& db, std::shared_ptr<Operator> op,
                       std::shared_ptr<Iterator> lhs,
                       size_t lhsIdx,
                       const std::unordered_set<Annotation> &rightAnno);
  virtual ~MaterializedSeedJoin() {}

  virtual bool next(std::vector<Match>& tuple) override;
  virtual void reset() override;
private:
  const DB& db;
  std::shared_ptr<Operator> op;

  std::shared_ptr<Iterator> left;
  size_t lhsIdx;
  const std::unordered_set<Annotation>& right;
  unsigned int minDistance;
  unsigned int maxDistance;

  std::unique_ptr<AnnoIt> matchesByOperator;
  std::vector<Match> currentLHSMatch;
  Match currentRHSMatch;
  bool currentMatchValid;
  std::list<Annotation> matchingRightAnnos;

  bool nextLeftMatch();
  bool nextRightAnnotation();

  bool checkReflexitivity(const nodeid_t& lhsNode, const Annotation& lhsAnno, const nodeid_t& rhsNode, const Annotation& rhsAnno)
  {
    if(!op->isReflexive() && lhsNode == rhsNode && checkAnnotationKeyEqual(lhsAnno, rhsAnno))
    {
      return false;
    }
    else
    {
      return true;
    }
  }

};



} // end namespace annis

