/*
   Copyright 2017 Thomas Krause <thomaskrause@posteo.de>

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

#include "plan.h"

#include <annis/annosearch/nodebyedgeannosearch.h>  // for NodeByEdgeAnnoSearch
#include <annis/db.h>                               // for DB
#include <annis/filter.h>                           // for Filter
#include <annis/join/indexjoin.h>                   // for IndexJoin
#include <annis/join/nestedloop.h>                  // for NestedLoopJoin
#include <annis/join/taskindexjoin.h>               // for TaskIndexJoin
#include <annis/join/threadindexjoin.h>             // for ThreadIndexJoin
#include <annis/join/threadnestedloop.h>            // for ThreadNestedLoop
#include <annis/operators/operator.h>               // for Operator
#include <annis/wrapper.h>                          // for ConstAnnoWrapper
#include <boost/container/vector.hpp>               // for operator!=
#include <cstdint>                                  // for uint64_t, int64_t
#include <map>                                      // for _Rb_tree_iterator
#include <memory>                                   // for shared_ptr, __sha...
#include <set>                                      // for set
#include <unordered_set>                            // for unordered_set
#include "annis/annosearch/annotationsearch.h"      // for EstimatedSearch
#include "annis/annostorage.h"                      // for AnnoStorage
#include "annis/iterators.h"                        // for Iterator
#include "annis/queryconfig.h"                      // for QueryConfig

using namespace annis;

Plan::Plan(std::shared_ptr<ExecutionNode> root)
  : root(root)
{
}

Plan::Plan(const Plan& orig)
{
  root = orig.root;
}

std::shared_ptr<ExecutionNode> Plan::join(std::shared_ptr<Operator> op,
    size_t lhsNodeNr, size_t rhsNodeNr,
    std::shared_ptr<ExecutionNode> lhs, std::shared_ptr<ExecutionNode> rhs,
    const DB& db,
    bool forceNestedLoop,
    size_t numOfBackgroundTasks,
    QueryConfig config)
{
  
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
  else if(config.avoidNestedBySwitch && !forceNestedLoop
    && op->isCommutative()
    && lhs->type == ExecutionNodeType::base)
  {
    // avoid a nested loop join by switching the operands
    std::shared_ptr<ExecutionNode> tmp = lhs;
    lhs = rhs;
    rhs = tmp;
    
    size_t tmpNodeID = lhsNodeNr;
    lhsNodeNr = rhsNodeNr;
    rhsNodeNr = tmpNodeID;
    
    type = ExecutionNodeType::seed;
  }
  
  std::shared_ptr<ExecutionNode> result = std::make_shared<ExecutionNode>();
  auto mappedPosLHS = lhs->nodePos.find(lhsNodeNr);
  auto mappedPosRHS = rhs->nodePos.find(rhsNodeNr);
  
  // make sure both source nodes are contained in the previous execution nodes
  if(mappedPosLHS == lhs->nodePos.end() || mappedPosRHS == rhs->nodePos.end())
  {
    // TODO: throw error?
    return result;
  }
  
  // create the join iterator
  
  std::shared_ptr<Iterator> join;
  if(type == ExecutionNodeType::filter)
  {
    result->type = ExecutionNodeType::filter;
    join = std::make_shared<Filter>(op, lhs->join, mappedPosLHS->second, mappedPosRHS->second);
  }
  else if(type == ExecutionNodeType::seed)
  {
    result->type = ExecutionNodeType::seed;
    result->numOfBackgroundTasks = numOfBackgroundTasks;
      
    std::shared_ptr<Iterator> rightIt = rhs->join;

    std::shared_ptr<ConstAnnoWrapper> constWrapper =
        std::dynamic_pointer_cast<ConstAnnoWrapper>(rightIt);
    if(constWrapper)
    {
      rightIt = constWrapper->getDelegate();
    }

    std::shared_ptr<EstimatedSearch> estSearch =
        std::dynamic_pointer_cast<EstimatedSearch>(rightIt);

    if(estSearch)
    {
      if(numOfBackgroundTasks > 0)
      {
        join = std::make_shared<ThreadIndexJoin>(lhs->join, mappedPosLHS->second, op,
                                                 createSearchFilter(db, estSearch),
                                                 numOfBackgroundTasks,
                                                 config.threadPool);
      }
      else if(config.enableTaskIndexJoin && config.threadPool)
      {
        join = std::make_shared<TaskIndexJoin>(lhs->join, mappedPosLHS->second, op,
                                               createSearchFilter(db, estSearch), 128, config.threadPool);
      }
      else
      {
        join = std::make_shared<IndexJoin>(db, op, lhs->join,
                                           mappedPosLHS->second,
                                           createSearchFilter(db, estSearch),
                                           searchFilterReturnsMaximalOneAnno(estSearch));
      }
    }
    else
    {
      // fallback to nested loop
      result->type = nested_loop;
      join = std::make_shared<NestedLoopJoin>(op, lhs->join, rhs->join,
                                              mappedPosLHS->second, mappedPosRHS->second, true, true);
    }
  }
  else
  {
    result->type = ExecutionNodeType::nested_loop;
    result->numOfBackgroundTasks = numOfBackgroundTasks;
    
    auto leftEst = estimateTupleSize(lhs);
    auto rightEst = estimateTupleSize(rhs);
    
    bool leftIsOuter = leftEst->output <= rightEst->output;
    
    if(numOfBackgroundTasks > 0)
    {
      join = std::make_shared<ThreadNestedLoop>(op, lhs->join, rhs->join,
                                              mappedPosLHS->second, mappedPosRHS->second, leftIsOuter,
                                              numOfBackgroundTasks,
                                              config.threadPool);
    }
    else
    {
      join = std::make_shared<NestedLoopJoin>(op, lhs->join, rhs->join,
                                              mappedPosLHS->second, mappedPosRHS->second, true, leftIsOuter);
    }
  }
  
  result->join = join;
  result->op = op;
  result->componentNr = lhs->componentNr;
  result->lhs = lhs;
  result->description =  "#" + std::to_string(lhsNodeNr+1) + " "
    + op->description() + " #" + std::to_string(rhsNodeNr+1);
  
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
  return static_cast<double>(estimateTupleSize(root)->intermediateSum);
}

std::map<size_t, size_t> Plan::getOptimizedParallelizationMapping(const DB &db, QueryConfig config)
{
  std::map<size_t, size_t> mapping;

  for(size_t available = config.numOfBackgroundTasks; available > 0; available -= 2)
  {
    std::pair<std::shared_ptr<ExecutionNode>, uint64_t> largest = findLargestProcessedInStep(
          root, config.enableThreadIndexJoin);
    if(largest.first)
    {
      if(mapping.find(largest.first->operatorIdx) == mapping.end())
      {
        mapping[largest.first->operatorIdx] = 2;
      }
      else
      {
        mapping[largest.first->operatorIdx] += 2;
      }
    }
    else
    {
      // nothing to optimize
      break;
    }
  }

  return mapping;
}

std::shared_ptr<ExecutionEstimate> Plan::estimateTupleSize(std::shared_ptr<ExecutionNode> node)
{
  static const std::uint64_t defaultBaseTuples = 100000;
  static const double defaultSelectivity = 0.1;
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
        std::int64_t guess = baseEstimate->guessMaxCount();
        if (guess >= 0)
        {
          node->estimate = std::make_shared<ExecutionEstimate>((std::uint64_t) guess, 0, 0);
          return node->estimate;
        } 
        else
        {
          node->estimate = std::make_shared<ExecutionEstimate>(defaultBaseTuples, 0, 0);
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
        long double operatorSelectivity = defaultSelectivity;
        if(node->op)
        {
          selectivity = operatorSelectivity = node->op->selectivity();
          double edgeAnnoSelectivity = node->op->edgeAnnoSelectivity();
          if(edgeAnnoSelectivity >= 0.0)
          {
            selectivity = selectivity * edgeAnnoSelectivity;
          }
        }
        
        std::uint64_t outputSize = static_cast<std::uint64_t>(((long double) estLHS->output) * ((long double) estRHS->output) * ((long double) selectivity));
        if(outputSize < 1)
        {
          // always assume at least one output item otherwise very small selectivity can fool the planner
          outputSize = 1;
        }
        std::uint64_t processedInStep;

        if (node->type == ExecutionNodeType::nested_loop)
        {
          if(estLHS->output < estRHS->output)
          {
            // we use LHS as outer
            processedInStep = estLHS->output + (estLHS->output * estRHS->output);
          }
          else
          {
            // we use RHS as outer
            processedInStep = estRHS->output + (estRHS->output * estLHS->output);
          }
        } 
        else if (node->type == ExecutionNodeType::seed)
        {
          // A index join processes each LHS and for each LHS the number of reachable nodes given by the operator.
          // The selectivity of the operator itself an estimation how many nodes are filtered out by the cross product.
          // We can use this number (without the edge annotation selectivity) to re-construct the number of reachable nodes.

          // avgReachable = (sel * cross) / lhs
          //              = (sel * lhs * rhs) / lhs
          //              = sel * rhs
          // processedInStep = lhs + (avgReachable * lhs)
          //                 = lhs + (sel * rhs * lhs)


          processedInStep =
              static_cast<std::uint64_t>(
                (long double) estLHS->output
                + (operatorSelectivity * (long double) estRHS->output * (long double) estLHS->output)
              );
        } 
        else
        {
          processedInStep = estLHS->output;
        }

        // return the output of this node and the sum of all intermediate results
        node->estimate = 
          std::make_shared<ExecutionEstimate>(outputSize,
                                              processedInStep + estLHS->intermediateSum + estRHS->intermediateSum,
                                              processedInStep);
        return node->estimate;

      }
      else if (node->lhs)
      { 
        // this is a filter node, the estimated number of of tuple is
        // count(lhs) * selectivity(op)
        auto estLHS = estimateTupleSize(node->lhs);
        double selectivity = defaultSelectivity;
        if(node->op)
        {
          selectivity = node->op->selectivity();
        }

        std::uint64_t processedInStep = estLHS->output;
        std::uint64_t outputSize = static_cast<std::uint64_t>(((double) estLHS->output) * selectivity);
       
        // return the output of this node and the sum of all intermediate results
        node->estimate = 
          std::make_shared<ExecutionEstimate>(outputSize, processedInStep + estLHS->intermediateSum, processedInStep);
        return node->estimate;

      }
    } // end if no cached estimate given
  }
  else
  {
    // a non-existing node doesn't have any cost
    node->estimate =  std::make_shared<ExecutionEstimate>(0, 0, 0);
    return node->estimate;
  }
  
  // we don't know anything about this node, return some large estimate
  // TODO: use DB do get a number relative to the overall number of nodes/annotations
  node->estimate = std::make_shared<ExecutionEstimate>(defaultBaseTuples, defaultBaseTuples, defaultBaseTuples);
  return node->estimate;
}

std::function<std::list<Annotation> (nodeid_t)> Plan::createSearchFilter(const DB &db, std::shared_ptr<EstimatedSearch> search)
{
  std::shared_ptr<ConstAnnoWrapper> constWrapper = std::dynamic_pointer_cast<ConstAnnoWrapper>(search);
  boost::optional<Annotation> constAnno;
  if(constWrapper)
  {
    search = constWrapper->getDelegate();
    constAnno = constWrapper->getConstAnno();
  }

  std::shared_ptr<AnnotationSearch> annoSearch = std::dynamic_pointer_cast<AnnotationSearch>(search);
  if(annoSearch)
  {
    return createAnnotationSearchFilter(db, annoSearch, constAnno);
  }
  std::shared_ptr<AnnotationKeySearch> annoKeySearch = std::dynamic_pointer_cast<AnnotationKeySearch>(search);
  if(annoKeySearch)
  {
    return createAnnotationKeySearchFilter(db, annoKeySearch, constAnno);
  }

  std::shared_ptr<NodeByEdgeAnnoSearch> byEdgeAnno = std::dynamic_pointer_cast<NodeByEdgeAnnoSearch>(search);
  if(byEdgeAnno)
  {
    return byEdgeAnno->getNodeAnnoMatchGenerator();
  }
  return [](nodeid_t) -> std::list<Annotation>  {return std::list<Annotation>();};
}

bool Plan::searchFilterReturnsMaximalOneAnno(std::shared_ptr<EstimatedSearch> search)
{
  std::shared_ptr<AnnotationSearch> annoSearch = std::dynamic_pointer_cast<AnnotationSearch>(search);
  if(annoSearch)
  {
    return annoSearch->getValidAnnotations().size() <= 1;
  }
  std::shared_ptr<AnnotationKeySearch> annoKeySearch = std::dynamic_pointer_cast<AnnotationKeySearch>(search);
  if(annoKeySearch)
  {
    return annoKeySearch->getValidAnnotationKeys().size() <= 1;
  }
  std::shared_ptr<NodeByEdgeAnnoSearch> byEdgeAnno = std::dynamic_pointer_cast<NodeByEdgeAnnoSearch>(search);
  if(byEdgeAnno)
  {
    return byEdgeAnno->maximalOneNodeAnno;
  }

  return false;
}

std::list<std::shared_ptr<ExecutionNode>> Plan::getDescendentNestedLoops(std::shared_ptr<ExecutionNode> node)
{
  std::list<std::shared_ptr<ExecutionNode>> result;
  if(node)
  {
    if(node->type == ExecutionNodeType::nested_loop)
    {
      result.push_back(node);
    }
    
    if(node->lhs) 
    {
      auto lhsNestedLoops = getDescendentNestedLoops(node->lhs);
      if(!lhsNestedLoops.empty())
      {
        for(auto nl : lhsNestedLoops)
        {
          result.push_back(nl);
        }
      }
    }
    
    if(node->rhs) 
    {
      auto rhsNestedLoops = getDescendentNestedLoops(node->rhs);
      if(!rhsNestedLoops.empty())
      {
        for(auto nl : rhsNestedLoops)
        {
          result.push_back(nl);
        }
      }
    }
  }
  return result;
}

std::function<std::list<Annotation> (nodeid_t)> Plan::createAnnotationSearchFilter(
    const DB& db,
    std::shared_ptr<AnnotationSearch> annoSearch, boost::optional<Annotation> constAnno)
{
  const std::unordered_set<Annotation>& validAnnos = annoSearch->getValidAnnotations();
  if(validAnnos.size() == 1)
  {
    const auto& rightAnno = *(validAnnos.begin());

    // no further checks required
    return [&db, rightAnno, constAnno](nodeid_t rhsNode) -> std::list<Annotation>
    {
      std::list<Annotation> result;
      auto foundAnno =
          db.nodeAnnos.getAnnotations(rhsNode, rightAnno.ns, rightAnno.name);

      if(!foundAnno.empty() && foundAnno[0].val == rightAnno.val)
      {
        if(constAnno)
        {
          result.push_back(*constAnno);
        }
        else
        {
          result.push_back(foundAnno[0]);
        }
      }

      return std::move(result);
    };
  }
  else
  {
    return [&db, validAnnos, constAnno](nodeid_t rhsNode) -> std::list<Annotation>
    {
      std::list<Annotation> result;
      // check all annotations which of them matches
      std::vector<Annotation> annos = db.nodeAnnos.getAnnotations(rhsNode);
      for(const auto& a : annos)
      {
        if(validAnnos.find(a) != validAnnos.end())
        {
          if(constAnno)
          {
            result.push_back(*constAnno);
          }
          else
          {
            result.push_back(a);
          }
        }
      }

      return std::move(result);
    };
  }
}


std::function<std::list<Annotation> (nodeid_t)> Plan::createAnnotationKeySearchFilter(const DB& db,
    std::shared_ptr<AnnotationKeySearch> annoKeySearch, boost::optional<Annotation> constAnno)
{
  const std::set<AnnotationKey>& validAnnoKeys = annoKeySearch->getValidAnnotationKeys();
  if(validAnnoKeys.size() == 1)
  {
    const auto& rightAnnoKey = *(validAnnoKeys.begin());

    // no further checks required
    return [&db, rightAnnoKey, constAnno](nodeid_t rhsNode) -> std::list<Annotation>
    {
      std::list<Annotation> result;
      auto foundAnno =
          db.nodeAnnos.getAnnotations(rhsNode, rightAnnoKey.ns, rightAnnoKey.name);

      if(!foundAnno.empty())
      {
        if(constAnno)
        {
          result.push_back(*constAnno);
        }
        else
        {
          result.push_back(foundAnno[0]);
        }

      }

      return std::move(result);
    };
  }
  else
  {
    return [&db, validAnnoKeys, constAnno](nodeid_t rhsNode) -> std::list<Annotation>
    {
      std::list<Annotation> result;
      // check all annotation keys
      for(AnnotationKey key : validAnnoKeys)
      {
       auto found = db.nodeAnnos.getAnnotations(rhsNode, key.ns, key.name);
       if(!found.empty())
       {
         if(constAnno)
         {
           result.push_back(*constAnno);
         }
         else
         {
          result.push_back(found[0]);
         }
       }
      }
      return std::move(result);
    };
  }
}

std::pair<std::shared_ptr<ExecutionNode>, uint64_t> Plan::findLargestProcessedInStep(
    std::shared_ptr<ExecutionNode> node, bool includeSeed)
{

  if(!node)
  {
    // nothing to find
    return {std::shared_ptr<ExecutionNode>(), 0u};
  }


  auto largestLHS = findLargestProcessedInStep(node->lhs);
  auto largestRHS = findLargestProcessedInStep(node->rhs);

  auto result = largestLHS;
  if(largestLHS.second < largestRHS.second)
  {
    result = largestRHS;
  }

  if(node->type == ExecutionNodeType::nested_loop || (includeSeed && node->type == ExecutionNodeType::seed))
  {
    std::shared_ptr<ExecutionEstimate> estNode = estimateTupleSize(node);

    // divide by the number of parallel processes
    uint64_t processedInSeed = estNode->processedInStep / (node->numOfBackgroundTasks == 0 ? 1 : node->numOfBackgroundTasks);
    if(result.second < processedInSeed)
    {
      result = {node, processedInSeed};
    }
  }

  return result;

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
  
  std::string result = indention + "+|";
  
  if(node->type == ExecutionNodeType::base)
  {
    // output the node number
    result += "#" + std::to_string(node->nodePos.begin()->first + 1);
    std::shared_ptr<EstimatedSearch> annoSearch = std::dynamic_pointer_cast<EstimatedSearch>(node->join);
    if(annoSearch)
    {
      std::string annoDebugString = annoSearch->debugString();
      if(!annoDebugString.empty())
      {
        result += ": " + annoDebugString;
      }
    }
  }
  else
  {
    result += typeToString(node->type);
  }
  result += "|";
  
  if(!node->description.empty())
  {
    result += "(" + node->description + ")";
  }
  
  if(node->estimate)
  {
    result +=  "[out: " 
      + std::to_string((std::uint64_t) node->estimate->output) 
      + " sum: " 
      + std::to_string((std::uint64_t) node->estimate->intermediateSum) 
      + " instep: "
      + std::to_string((std::uint64_t) node->estimate->processedInStep)
      + "]";
  }
  if(node->op)
  {
    result += "{sel: " + std::to_string(node->op->selectivity());
    if((node->type == ExecutionNodeType::seed || node->type == ExecutionNodeType::nested_loop)
       && node->numOfBackgroundTasks > 0)
    {
      result +=" tasks: " + std::to_string(node->numOfBackgroundTasks);
    }
    result += "}";
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

