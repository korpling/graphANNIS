/*
 * To change this license header, choose License Headers in Project Properties.
 * To change this template file, choose Tools | Templates
 * and open the template in the editor.
 */

/* 
 * File:   Plan.cpp
 * Author: thomas
 * 
 * Created on 1. März 2016, 11:48
 */

#include <annis/util/plan.h>
#include <map>
#include <memory>

#include <annis/db.h>
#include <annis/wrapper.h>
#include <annis/operators/operator.h>
#include <annis/join/nestedloop.h>
#include <annis/join/seed.h>
#include <annis/filter.h>

using namespace annis;

Plan::Plan(std::shared_ptr<ExecutionNode> root)
: root(root), cost(-1.0)
{
}

Plan::Plan(const Plan& orig)
{
  root = orig.root;
  cost = orig.cost;
}

std::shared_ptr<ExecutionNode> Plan::join(
    std::shared_ptr<Operator> op, 
    size_t lhsNode, size_t rhsNode,
    std::shared_ptr<ExecutionNode> lhs, std::shared_ptr<ExecutionNode> rhs,
    const DB& db,
    ExecutionNodeType type)
{
  std::shared_ptr<ExecutionNode> result = std::make_shared<ExecutionNode>();
  
  auto mappedPosLHS = lhs->nodePos.find(lhsNode);
  auto mappedPosRHS = rhs->nodePos.find(rhsNode);
  
  // make sure both source nodes are contained in the previous execution nodes
  if(mappedPosLHS == lhs->nodePos.end() || mappedPosRHS == rhs->nodePos.end()
    || lhs->componentNr != rhs->componentNr)
  {
    // TODO: throw error?
    return result;
  }
  
  // create the join iterator
  
  std::shared_ptr<Iterator> join;
  if(type == ExecutionNodeType::filter)
  {
    result->type = ExecutionNodeType::filter;
    join = std::make_shared<Filter>(op, lhs->join, rhs->join, mappedPosLHS->second, mappedPosRHS->second);
  }
  else if(type == ExecutionNodeType::seed)
  {
    result->type = ExecutionNodeType::seed;
      
    std::shared_ptr<Iterator> rightIt = rhs->join;

    std::shared_ptr<ConstAnnoWrapper> constWrapper =
        std::dynamic_pointer_cast<ConstAnnoWrapper>(rightIt);
    if(constWrapper)
    {
      rightIt = constWrapper->getDelegate();
    }

    std::shared_ptr<AnnotationKeySearch> keySearch =
        std::dynamic_pointer_cast<AnnotationKeySearch>(rightIt);
    std::shared_ptr<AnnotationSearch> annoSearch =
        std::dynamic_pointer_cast<AnnotationSearch>(rightIt);

    if(keySearch)
    {
      join = std::make_shared<AnnoKeySeedJoin>(db, op, lhs->join,
        mappedPosLHS->second,
        keySearch->getValidAnnotationKeys());
    }
    else if(annoSearch)
    {
      join = std::make_shared<MaterializedSeedJoin>(db, op, lhs->join,
        mappedPosLHS->second,
        annoSearch->getValidAnnotations());
    }
    else
    {
      // fallback to nested loop
      join = std::make_shared<NestedLoopJoin>(op, lhs->join, rhs->join, mappedPosLHS->second, mappedPosRHS->second);
    }
  }
  else
  {
    result->type = ExecutionNodeType::nested_loop;
    join = std::make_shared<NestedLoopJoin>(op, lhs->join, rhs->join, mappedPosLHS->second, mappedPosRHS->second);
  }
  
  result->join = join;
  result->componentNr = lhs->componentNr;
  result->lhs = lhs;
  result->rhs = rhs;
  
  // merge both node positions
  for(const auto& pos : lhs->nodePos)
  {
    result->nodePos.insert(pos);
  }
  // the RHS has an offset after the join
  size_t offset = lhs->nodePos.size();
  for(const auto& pos : rhs->nodePos)
  {
    result->nodePos.insert({pos.first, pos.second + offset});
  }
  
  return result;
}


bool Plan::executeStep(std::vector<Match>& result)
{
  if(root && root->join)
  {
    return root->join->next(result);
  }
  else
  {
    return false;
  }
}


double Plan::getCost() {
  if(cost < 0.0)
  {
    ExecutionEstimate est = estimateTupleSize(root);
    cost = est.intermediateSum;
  }
  
  return cost;
}

ExecutionEstimate Plan::estimateTupleSize(std::shared_ptr<ExecutionNode> node)
{
  static const double defaultBaseTuples = 100000.0;
  static const double defaultSelectivity = 0.5;
  if(node)
  {
    std::shared_ptr<EstimatedSearch> baseEstimate =
      std::dynamic_pointer_cast<EstimatedSearch>(node->join);
    if(baseEstimate)
    {
      // directly use the estimated search this exec node
      int guess = baseEstimate->guessMaxCount();
      if(guess >= 0)
      {
        return {(double) guess, (double) guess};
      }
      else
      {
        return {defaultBaseTuples, defaultBaseTuples};
      }
    }
    else if(node->lhs && node->rhs)
    {
      // this is a join node, the estimated number of of tuple is
      // (count(lhs) * count(rhs)) / selectivity(op)
      auto estLHS = estimateTupleSize(node->lhs);
      auto estRHS = estimateTupleSize(node->lhs);
      double selectivity = defaultSelectivity;
      // TODO: get the selectivity from the operator
      double output = ((estLHS.output * estRHS.output) / selectivity);
      
      // return the output of this node and the sum of all intermediate results
      return {output, output + estLHS.intermediateSum + estRHS.intermediateSum};
       
    }

  }
  else
  {
    // a non-existing node doesn't have any cost
    return {0.0, 0.0};
  }
  
  // we don't know anything about this node, return some large estimate
  // TODO: use DB do get a number relative to the overall number of nodes/annotations
  return {defaultBaseTuples, defaultBaseTuples};
}



Plan::~Plan()
{
}

