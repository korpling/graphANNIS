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
: root(root)
{
}

Plan::Plan(const Plan& orig)
{
  root = orig.root;
}

std::shared_ptr<ExecutionNode> Plan::join(
    std::shared_ptr<Operator> op, 
    size_t lhsNode, size_t rhsNode,
    std::shared_ptr<ExecutionNode> lhs, std::shared_ptr<ExecutionNode> rhs,
    const DB& db,
    bool forceNestedLoop)
{
  std::shared_ptr<ExecutionNode> result = std::make_shared<ExecutionNode>();
  
  auto mappedPosLHS = lhs->nodePos.find(lhsNode);
  auto mappedPosRHS = rhs->nodePos.find(rhsNode);
  
  // make sure both source nodes are contained in the previous execution nodes
  if(mappedPosLHS == lhs->nodePos.end() || mappedPosRHS == rhs->nodePos.end())
  {
    // TODO: throw error?
    return result;
  }
  
  ExecutionNodeType type = ExecutionNodeType::nested_loop;
  if(lhs->componentNr == rhs->componentNr)
  {
    type = ExecutionNodeType::filter;
  }
  else if(rhs->type == ExecutionNodeType::base && !forceNestedLoop)
  { 
    // if the right side is not another join we can use a seed join
    type = ExecutionNodeType::seed;
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
  if(type != ExecutionNodeType::filter)
  {
    // only set a rhs when this is an actual join
    result->rhs = rhs;
  }
  rhs->componentNr = result->componentNr;
  
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


double Plan::getCost() 
{
  // the estimation is cached in the root so multiple calls to getCost() won't do any harm
  return estimateTupleSize(root)->intermediateSum;
}

std::shared_ptr<ExecutionEstimate> Plan::estimateTupleSize(std::shared_ptr<ExecutionNode> node)
{
  static const std::uint64_t defaultBaseTuples = 100000;
  static const double defaultSelectivity = 0.5;
  if(node)
  {
    if (node->estimate)
    {
      // return the cached estimate
      return node->estimate;
    } 
    else
    {
      std::shared_ptr<EstimatedSearch> baseEstimate =
        std::dynamic_pointer_cast<EstimatedSearch>(node->join);
      if (baseEstimate)
      {
        // directly use the estimated search this exec node
        int guess = baseEstimate->guessMaxCount();
        if (guess >= 0)
        {
          node->estimate = std::make_shared<ExecutionEstimate>((std::uint64_t) guess, (std::uint64_t) guess);
          return node->estimate;
        } 
        else
        {
          node->estimate = std::make_shared<ExecutionEstimate>(defaultBaseTuples, defaultBaseTuples);
          return node->estimate;
        }
      } 
      else if (node->lhs && node->rhs)
      {
        // this is a join node, the estimated number of of tuple is
        // (count(lhs) * count(rhs)) * selectivity(op)
        auto estLHS = estimateTupleSize(node->lhs);
        auto estRHS = estimateTupleSize(node->rhs);
        double selectivity = defaultSelectivity;
        // TODO: get the selectivity from the operator

        std::uint64_t outLeftWithSelectivity = ((double) estLHS->output * selectivity);
        std::uint64_t outRightWithSelectivity = ((double) estRHS->output * selectivity);
        
        std::uint64_t outputSize = outLeftWithSelectivity * outRightWithSelectivity;
        std::uint64_t processedInStep;

        if (node->type == ExecutionNodeType::nested_loop)
        {
          processedInStep = estLHS->output + (estLHS->output * estRHS->output);
        } 
        else if (node->type == ExecutionNodeType::seed)
        {
          std::uint64_t x = (((double)(outputSize - estLHS->output)) / (double) estLHS->output);
          processedInStep = estLHS->output * x;
        } 
        else
        {
          processedInStep = estLHS->output;
        }

        // return the output of this node and the sum of all intermediate results
        node->estimate = 
          std::make_shared<ExecutionEstimate>(outputSize, processedInStep + estLHS->intermediateSum + estRHS->intermediateSum);
        return node->estimate;

      }
      else if (node->lhs)
      {
        // this is a filter node, the estimated number of of tuple is
        // count(lhs) * selectivity(op)
        auto estLHS = estimateTupleSize(node->lhs);
        double selectivity = defaultSelectivity;
        // TODO: get the selectivity from the operator

        std::uint64_t processedInStep = estLHS->output;
        std::uint64_t outputSize = ((double) estLHS->output) * selectivity;
       
        // return the output of this node and the sum of all intermediate results
        node->estimate = 
          std::make_shared<ExecutionEstimate>(outputSize, processedInStep + estLHS->intermediateSum);
        return node->estimate;

      }
    } // end if no cached estimate given
  }
  else
  {
    // a non-existing node doesn't have any cost
    node->estimate =  std::make_shared<ExecutionEstimate>(0, 0);
    return node->estimate;
  }
  
  // we don't know anything about this node, return some large estimate
  // TODO: use DB do get a number relative to the overall number of nodes/annotations
  node->estimate = std::make_shared<ExecutionEstimate>(defaultBaseTuples, defaultBaseTuples);
  return node->estimate;
}

void Plan::clearCachedEstimate(std::shared_ptr<ExecutionNode> node) 
{
  if(node)
  {
    node->estimate.reset();
    
    if(node->lhs)
    {
      clearCachedEstimate(node->lhs);
    }

    if(node->rhs)
    {
      clearCachedEstimate(node->rhs);
    }
  }
}

std::string Plan::debugString() const
{
  return debugStringForNode(root, "");
}

std::string Plan::debugStringForNode(std::shared_ptr<const ExecutionNode> node, std::string indention) const
{
  if(!node)
  {
    return "";
  }
  
  std::string result = indention + "(";
  
  if(node->type == ExecutionNodeType::base)
  {
    // output the node number
    result += "#" + std::to_string(node->nodePos.begin()->first + 1);
  }
  else
  {
    result += typeToString(node->type);
  }
  result += ")";
  
  if(node->estimate)
  {
    result +=  "[out: " 
      + std::to_string((std::uint64_t) node->estimate->output) 
      + " sum: " 
      + std::to_string((std::uint64_t) node->estimate->intermediateSum) 
      + "]";
  }
  
  result += "\n";
  
  if(node->lhs)
  {
    result += debugStringForNode(node->lhs, indention + "    ");
  }
  if(node->rhs)
  {
    result += debugStringForNode(node->rhs, indention + "    ");
  }
  
  return result;
}

std::string Plan::typeToString(ExecutionNodeType type) const
{
  switch(type)
  {
    case ExecutionNodeType::base:
      return "base";
    case ExecutionNodeType::filter:
      return "filter";
    case ExecutionNodeType::nested_loop:
      return "nested_loop";
    case ExecutionNodeType::seed:
      return "seed";
    default:
      return "<unknown>";
  }

}


Plan::~Plan()
{
}

