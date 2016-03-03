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
#include <memory>
#include <vector>
#include <map>

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
  : output(0), intermediateSum(0)
  {}
  
  ExecutionEstimate(std::uint64_t output, std::uint64_t intermediateSum)
    : output(output), intermediateSum(intermediateSum)
  {}
  
  std::uint64_t output;
  std::uint64_t intermediateSum;
};

struct ExecutionNode
{
  ExecutionNodeType type;
  
  std::shared_ptr<Iterator> join;
  std::map<size_t, size_t> nodePos;
  int componentNr;
  
  std::shared_ptr<ExecutionNode> lhs;
  std::shared_ptr<ExecutionNode> rhs;
  
  std::shared_ptr<ExecutionEstimate> estimate;
};


class Plan
{
public:
  Plan(std::shared_ptr<ExecutionNode> root);
  
  Plan(const Plan& orig);
  virtual ~Plan();
  
  bool executeStep(std::vector<Match>& result);
  double getCost();
  
  static std::shared_ptr<ExecutionNode> join(
    std::shared_ptr<Operator> op, 
    size_t lhsNode, size_t rhsNode,
    std::shared_ptr<ExecutionNode>, std::shared_ptr<ExecutionNode> rhs,
    const DB& db,
    bool forceNestedLoop);
  
  std::string debugString() const;
  
private:
  std::shared_ptr<ExecutionNode> root;
  
private:
  std::shared_ptr<ExecutionEstimate> estimateTupleSize(std::shared_ptr<ExecutionNode> node);
  void clearCachedEstimate(std::shared_ptr<ExecutionNode> node);
  
  std::string debugStringForNode(std::shared_ptr<const ExecutionNode> node, std::string indention) const;
  std::string typeToString(ExecutionNodeType type) const;
};

} // end namespace annis