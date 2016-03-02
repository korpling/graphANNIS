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

Plan::Plan(const ExecutionNode& root)
: root(root), cost(-1.0)
{
}

Plan::Plan(const Plan& orig)
{
  root = orig.root;
  cost = orig.cost;
}

ExecutionNode Plan::join(
    std::shared_ptr<Operator> op, 
    size_t lhsNode, size_t rhsNode,
    const ExecutionNode& lhs, const ExecutionNode& rhs,
    const DB& db,
    ExecutionNodeType type)
{
  ExecutionNode result;
  
  auto mappedPosLHS = lhs.nodePos.find(lhsNode);
  auto mappedPosRHS = rhs.nodePos.find(rhsNode);
  
  // make sure both source nodes are contained in the previous execution nodes
  if(mappedPosLHS == lhs.nodePos.end() || mappedPosRHS == rhs.nodePos.end()
    || lhs.componentNr != rhs.componentNr)
  {
    // TODO: throw error?
    return result;
  }
  
  // create the join iterator
  
  std::shared_ptr<Iterator> join;
  if(type == ExecutionNodeType::filter)
  {
    result.type = ExecutionNodeType::filter;
    join = std::make_shared<Filter>(op, lhs.join, rhs.join, mappedPosLHS->second, mappedPosRHS->second);
  }
  else if(type == ExecutionNodeType::seed)
  {
    result.type = ExecutionNodeType::seed;
      
    std::shared_ptr<Iterator> rightIt = rhs.join;

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
      join = std::make_shared<AnnoKeySeedJoin>(db, op, lhs.join,
        mappedPosRHS->second,
        keySearch->getValidAnnotationKeys());
    }
    else if(annoSearch)
    {
      join = std::make_shared<MaterializedSeedJoin>(db, op, lhs.join,
        mappedPosRHS->second,
        annoSearch->getValidAnnotations());
    }
    else
    {
      // fallback to nested loop
      join = std::make_shared<NestedLoopJoin>(op, lhs.join, rhs.join, mappedPosLHS->second, mappedPosRHS->second);
    }
  }
  else
  {
    result.type = ExecutionNodeType::nested_loop;
    join = std::make_shared<NestedLoopJoin>(op, lhs.join, rhs.join, mappedPosLHS->second, mappedPosRHS->second);
  }
  
  result.join = join;
  
  // merge both node positions
  for(const auto& pos : lhs.nodePos)
  {
    result.nodePos.insert(pos);
  }
  // the RHS has an offset after the join
  size_t offset = lhs.nodePos.size();
  for(const auto& pos : lhs.nodePos)
  {
    result.nodePos.insert({pos.first, pos.second + offset});
  }
  
  return result;
}


bool Plan::executeStep(std::vector<Match>& result)
{
  if(root.join)
  {
    return root.join->next(result);
  }
  else
  {
    return false;
  }
}


double Plan::getCost() {
  if(cost < 0.0)
  {
    // TODO: calculate the cost
  }
  
  return cost;
}


Plan::~Plan()
{
}

