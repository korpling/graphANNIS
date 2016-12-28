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

/**
 * A join that takes the left argument as a seed, finds all connected nodes
 * (probably using and index of the graph storage) and checks the condition for each node.
 * This join is not parallized.
 */
class IndexJoin : public Iterator
{
public:
  IndexJoin(const DB& db, std::shared_ptr<Operator> op,
           std::shared_ptr<Iterator> lhs,
            size_t lhsIdx,
           std::function< std::list<Match> (nodeid_t) > matchGeneratorFunc);
  virtual ~IndexJoin();

  virtual bool next(std::vector<Match>& tuple) override;
  virtual void reset() override;
private:
  const DB& db;
  std::shared_ptr<Operator> op;

  std::shared_ptr<Iterator> left;
  const size_t lhsIdx;
  const std::function<std::list<Match> (nodeid_t)> matchGeneratorFunc;

  std::unique_ptr<AnnoIt> matchesByOperator;
  std::vector<Match> currentLHSMatch;
  bool currentLHSMatchValid;
  std::list<Match> rhsCandidates;

  Match currentRHSMatch;


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

