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
};

class Plan
{
public:
  Plan(const ExecutionNode& root);
  
  Plan(const Plan& orig);
  virtual ~Plan();
  
  bool executeStep(std::vector<Match>& result);
  double getCost();
  
  static ExecutionNode join(
    std::shared_ptr<Operator> op, 
    size_t lhsNode, size_t rhsNode,
    const ExecutionNode& lhs, const ExecutionNode& rhs,
    const DB& db,
    ExecutionNodeType type);
  
private:
  ExecutionNode root;
  double cost;
  
};

} // end namespace annis