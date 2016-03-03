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
  
struct ExecutionNode
{
  ExecutionNodeType type;
  
  std::shared_ptr<Iterator> join;
  std::map<size_t, size_t> nodePos;
  int componentNr;
  
  std::shared_ptr<ExecutionNode> lhs;
  std::shared_ptr<ExecutionNode> rhs;
};

struct ExecutionEstimate
{
  double output;
  double intermediateSum;
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
    ExecutionNodeType type);
  
private:
  std::shared_ptr<ExecutionNode> root;
  double cost;
  
private:
  ExecutionEstimate estimateTupleSize(std::shared_ptr<ExecutionNode> node);
  
};

} // end namespace annis