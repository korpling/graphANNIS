/* 
 * File:   plan.h
 * Author: thomas
 *
 * Created on 1. März 2016, 11:48
 */

#pragma once

#include <annis/db.h>
#include <annis/iterators.h>
#include <annis/operators/operator.h>
#include <annis/annosearch/annotationsearch.h>
#include <annis/queryconfig.h>
#include <memory>
#include <vector>
#include <map>

#include <annis/util/threadpool.h>

namespace annis
{

enum ExecutionNodeType
{
  base,
  nested_loop,
  seed,
  filter,
  num_of_ExecutionNodeType
};


struct ExecutionEstimate
{
  ExecutionEstimate()
  : output(0), intermediateSum(0), processedInStep(0)
  {}
  
  ExecutionEstimate(std::uint64_t output, std::uint64_t intermediateSum, std::uint64_t processedInStep)
    : output(output), intermediateSum(intermediateSum), processedInStep(processedInStep)
  {}
  
  const std::uint64_t output;
  const std::uint64_t intermediateSum;
  const std::uint64_t processedInStep;
};

struct ExecutionNode
{
  ExecutionNodeType type;
  
  std::shared_ptr<Iterator> join;  
  std::shared_ptr<Operator> op;
  std::map<size_t, size_t> nodePos;
  size_t componentNr;
  /** Only valid for seed join types */
  size_t numOfBackgroundTasks;
  
  std::shared_ptr<ExecutionNode> lhs;
  std::shared_ptr<ExecutionNode> rhs;

  size_t lhsNodeNr;
  size_t rhsNodeNr;

  std::shared_ptr<ExecutionEstimate> estimate;
  
  std::string description;
};


class Plan
{
public:
  Plan(std::shared_ptr<ExecutionNode> root);
  
  Plan(const Plan& orig);
  virtual ~Plan();
  
  bool executeStep(std::vector<Match>& result);
  double getCost();

  void optimizeParallelization(const DB &db, QueryConfig config);
  
  static std::shared_ptr<ExecutionNode> join(std::shared_ptr<Operator> op,
    size_t lhsNodeNr, size_t rhsNodeNr,
    std::shared_ptr<ExecutionNode>, std::shared_ptr<ExecutionNode> rhs,
    const DB& db,
    bool forceNestedLoop,
    QueryConfig config);
  
  std::string debugString() const;
  
  bool hasNestedLoop() const;

  static std::function<std::list<Annotation> (nodeid_t)> createSearchFilter(const DB& db,
    std::shared_ptr<EstimatedSearch> search);

  static bool searchFilterReturnsMaximalOneAnno(std::shared_ptr<EstimatedSearch> search);
  
private:
  std::shared_ptr<ExecutionNode> root;
  
private:
  static std::shared_ptr<ExecutionEstimate> estimateTupleSize(std::shared_ptr<ExecutionNode> node);
  static void clearCachedEstimate(std::shared_ptr<ExecutionNode> node);
  
  std::string debugStringForNode(std::shared_ptr<const ExecutionNode> node, std::string indention) const;
  std::string typeToString(ExecutionNodeType type) const;
  
  static bool descendendantHasNestedLoop(std::shared_ptr<ExecutionNode> node);

  static std::function<std::list<Annotation> (nodeid_t)> createAnnotationSearchFilter(
      const DB& db, std::shared_ptr<AnnotationSearch> annoSearch,
      boost::optional<Annotation> constAnno = boost::optional<Annotation>());

  static std::function<std::list<Annotation> (nodeid_t)> createAnnotationKeySearchFilter(
      const DB& db, std::shared_ptr<AnnotationKeySearch> annoKeySearch,
      boost::optional<Annotation> constAnno = boost::optional<Annotation>());

  static std::pair<std::shared_ptr<ExecutionNode>, uint64_t> findLargestProcessedInStep(std::shared_ptr<ExecutionNode> node);
};

} // end namespace annis
