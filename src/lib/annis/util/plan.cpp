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
#include <annis/filter/binaryfilter.h>              // for BinaryFilter
#include <annis/join/indexjoin.h>                   // for IndexJoin
#include <annis/join/nestedloop.h>                  // for NestedLoopJoin
#include <annis/join/taskindexjoin.h>               // for TaskIndexJoin
#include <annis/join/threadindexjoin.h>             // for ThreadIndexJoin
#include <annis/join/threadnestedloop.h>            // for ThreadNestedLoop
#ifdef ENABLE_SIMD_SUPPORT
  #include <annis/join/simdindexjoin.h>
#endif
#include <annis/operators/operator.h>               // for Operator
#include <annis/wrapper.h>                          // for ConstAnnoWrapper
#include <boost/container/vector.hpp>               // for operator!=
#include <cstdint>                                  // for uint64_t, int64_t
#include <map>                                      // for _Rb_tree_iterator
#include <memory>                                   // for shared_ptr, __sha...
#include <set>                                      // for set
#include <unordered_set>                            // for unordered_set
#include "annis/annosearch/estimatedsearch.h"      // for EstimatedSearch
#include <annis/annosearch/regexannosearch.h>
#include <annis/annosearch/exactannokeysearch.h>
#include <annis/annosearch/exactannovaluesearch.h>
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
    type = ExecutionNodeType::index_join;
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
    
    type = ExecutionNodeType::index_join;
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

  boost::optional<std::string> extraDescription;
  
  // create the join iterator
  
  std::shared_ptr<Iterator> join;
  if(type == ExecutionNodeType::filter)
  {
    result->type = ExecutionNodeType::filter;
    join = std::make_shared<BinaryFilter>(op, lhs->join, mappedPosLHS->second, mappedPosRHS->second);
  }
  else if(type == ExecutionNodeType::index_join)
  {
    result->type = ExecutionNodeType::index_join;
    result->numOfBackgroundTasks = numOfBackgroundTasks;
      
    std::shared_ptr<Iterator> rightIt = rhs->join;

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
      #ifdef ENABLE_SIMD_SUPPORT
      else if(config.enableSIMDIndexJoin
              && std::dynamic_pointer_cast<AnnotationSearch>(estSearch)
              && searchFilterReturnsMaximalOneAnno(estSearch))
      {
        const std::unordered_set<Annotation>& validAnnos
            = std::static_pointer_cast<AnnotationSearch>(estSearch)->getValidAnnotations();
        join = std::make_shared<SIMDIndexJoin>(lhs->join, mappedPosLHS->second, op, db.nodeAnnos, *validAnnos.begin() );
      }
      #endif // ENABLE_SIMD_SUPPORT
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
      result->type = ExecutionNodeType::nested_loop;
      result->numOfBackgroundTasks = numOfBackgroundTasks;

      auto leftEst = estimateTupleSize(lhs);
      auto rightEst = estimateTupleSize(rhs);

      bool leftIsOuter = leftEst->output <= rightEst->output;
      extraDescription = leftIsOuter ? "lhs->rhs" : "rhs->lhs";

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
  }
  else
  {
    result->type = ExecutionNodeType::nested_loop;
    result->numOfBackgroundTasks = numOfBackgroundTasks;
    
    auto leftEst = estimateTupleSize(lhs);
    auto rightEst = estimateTupleSize(rhs);
    
    bool leftIsOuter = leftEst->output <= rightEst->output;
    extraDescription = leftIsOuter ? "lhs->rhs" : "rhs->lhs";
    
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
  if(extraDescription)
  {
    result->description = result->description + " " + *extraDescription;
  }
  
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
    std::vector<Match> tmp;
    if(root->join->next(tmp))
    {
      // re-order the matched nodes by the original node position of the query
      result.resize(tmp.size());
      for(const auto& nodeMapping : root->nodePos)
      {
        result[nodeMapping.first] = tmp[nodeMapping.second];
      }
      return true;
    }
    else
    {
      return false;
    }

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

        std::uint64_t outputSize = 1;

        Operator::EstimationType estType = node->op->estimationType();
        long double operatorSelectivity = 1.0;

        if(estType == Operator::EstimationType::SELECTIVITY)
        {
          double selectivity = defaultSelectivity;
          operatorSelectivity = defaultSelectivity;
          if(node->op)
          {
            selectivity = operatorSelectivity = node->op->selectivity();
            double edgeAnnoSelectivity = node->op->edgeAnnoSelectivity();
            if(edgeAnnoSelectivity >= 0.0)
            {
              selectivity = selectivity * edgeAnnoSelectivity;
            }
            outputSize =
              static_cast<std::uint64_t>(((long double) estLHS->output) * ((long double) estRHS->output) * ((long double) selectivity));
          }
        }
        else if(estType == Operator::EstimationType::MIN)
        {
          outputSize = std::min(estLHS->output, estRHS->output);
        }
        else if(estType == Operator::EstimationType::MAX)
        {
          outputSize = std::max(estLHS->output, estRHS->output);
        }

        if(outputSize < 1)
        {
          // always assume at least one output item otherwise very small selectivity can fool the planner
          outputSize = 1;
        }
        std::uint64_t processedInStep;

        if (node->type == ExecutionNodeType::nested_loop)
        {
          processedInStep = calculateNestedLoopProcessed(estLHS->output, estRHS->output);
        } 
        else if (node->type == ExecutionNodeType::index_join
                 && node->op->estimationType() == Operator::EstimationType::SELECTIVITY)
        {
          processedInStep = calculateIndexJoinProcessed(operatorSelectivity, estLHS->output, estRHS->output);
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
  boost::optional<Annotation> constAnno = search->getConstAnnoValue();

  std::shared_ptr<RegexAnnoSearch> regexSearch = std::dynamic_pointer_cast<RegexAnnoSearch>(search);
  if(regexSearch)
  {
    return createRegexAnnoSearchFilter(db, regexSearch, constAnno);
  }

  std::shared_ptr<ExactAnnoValueSearch> annoSearch = std::dynamic_pointer_cast<ExactAnnoValueSearch>(search);
  if(annoSearch)
  {
    return createAnnotationSearchFilter(db, annoSearch, constAnno);
  }
  std::shared_ptr<ExactAnnoKeySearch> annoKeySearch = std::dynamic_pointer_cast<ExactAnnoKeySearch>(search);
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
  std::shared_ptr<RegexAnnoSearch> regexSearch = std::dynamic_pointer_cast<RegexAnnoSearch>(search);
  if(regexSearch)
  {
    return false;
  }

  std::shared_ptr<ExactAnnoValueSearch> annoSearch = std::dynamic_pointer_cast<ExactAnnoValueSearch>(search);
  if(annoSearch)
  {
    return annoSearch->getValidAnnotations().size() <= 1;
  }
  std::shared_ptr<ExactAnnoKeySearch> annoKeySearch = std::dynamic_pointer_cast<ExactAnnoKeySearch>(search);
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

std::function<std::list<Annotation> (nodeid_t)> Plan::createAnnotationSearchFilter(const DB& db,
    std::shared_ptr<ExactAnnoValueSearch> annoSearch, boost::optional<Annotation> constAnno)
{
  const std::unordered_set<Annotation>& validAnnos = annoSearch->getValidAnnotations();
  auto outputFilter = annoSearch->getOutputFilter();

  if(validAnnos.size() == 1)
  {
    const auto& rightAnno = *(validAnnos.begin());

    // no further checks required
    return [&db, rightAnno, constAnno, outputFilter](nodeid_t rhsNode) -> std::list<Annotation>
    {
      std::list<Annotation> result;
      auto foundAnno =
          db.nodeAnnos.getAnnotations(rhsNode, rightAnno.ns, rightAnno.name);

      if(foundAnno && foundAnno->val == rightAnno.val && outputFilter({rhsNode, *foundAnno}))
      {
        if(constAnno)
        {
          result.push_back(*constAnno);
        }
        else
        {
          result.push_back(*foundAnno);
        }
      }

      return std::move(result);
    };
  }
  else
  {
    return [&db, validAnnos, constAnno, outputFilter](nodeid_t rhsNode) -> std::list<Annotation>
    {
      std::list<Annotation> result;
      // check all annotations which of them matches
      std::vector<Annotation> annos = db.nodeAnnos.getAnnotations(rhsNode);
      for(const auto& a : annos)
      {
        if(validAnnos.find(a) != validAnnos.end() && outputFilter({rhsNode, a}))
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

std::function<std::list<Annotation> (nodeid_t)> Plan::createRegexAnnoSearchFilter(
    const DB &db, std::shared_ptr<RegexAnnoSearch> regexSearch, boost::optional<Annotation> constAnno)
{

  auto outputFilter = regexSearch->getOutputFilter();
  const std::set<AnnotationKey>& validAnnoKeys = regexSearch->getValidAnnotationKeys();

  if(validAnnoKeys.size() == 1)
  {
    const auto& rightAnnoKey = *(validAnnoKeys.begin());

    return [&db, rightAnnoKey, constAnno, outputFilter, regexSearch](nodeid_t rhsNode) -> std::list<Annotation>
    {
      std::list<Annotation> result;
      auto foundAnno =
          db.nodeAnnos.getAnnotations(rhsNode, rightAnnoKey.ns, rightAnnoKey.name);

      if(foundAnno && regexSearch->valueMatches(db.strings.str(foundAnno->val)) && outputFilter({rhsNode, *foundAnno}))
      {
        if(constAnno)
        {
          result.push_back(*constAnno);
        }
        else
        {
          result.push_back(*foundAnno);
        }

      }

      return std::move(result);
    };
  }
  else
  {
    return [&db, validAnnoKeys, constAnno, outputFilter, regexSearch](nodeid_t rhsNode) -> std::list<Annotation>
    {
      std::list<Annotation> result;
      // check all annotation keys
      for(AnnotationKey key : validAnnoKeys)
      {
       auto found = db.nodeAnnos.getAnnotations(rhsNode, key.ns, key.name);
       if(found && regexSearch->valueMatches(db.strings.str(found->val)) && outputFilter({rhsNode, *found}))
       {
         if(constAnno)
         {
           result.push_back(*constAnno);
         }
         else
         {
          result.push_back(*found);
         }
       }
      }
      return std::move(result);
    };
  }
}


std::function<std::list<Annotation> (nodeid_t)> Plan::createAnnotationKeySearchFilter(const DB& db,
    std::shared_ptr<ExactAnnoKeySearch> annoKeySearch, boost::optional<Annotation> constAnno)
{
  const std::set<AnnotationKey>& validAnnoKeys = annoKeySearch->getValidAnnotationKeys();
  auto outputFilter = annoKeySearch->getOutputFilter();

  if(validAnnoKeys.size() == 1)
  {
    const auto& rightAnnoKey = *(validAnnoKeys.begin());

    // no further checks required
    return [&db, rightAnnoKey, constAnno, outputFilter](nodeid_t rhsNode) -> std::list<Annotation>
    {
      std::list<Annotation> result;
      auto foundAnno =
          db.nodeAnnos.getAnnotations(rhsNode, rightAnnoKey.ns, rightAnnoKey.name);

      if(foundAnno && outputFilter({rhsNode, *foundAnno}))
      {
        if(constAnno)
        {
          result.push_back(*constAnno);
        }
        else
        {
          result.push_back(*foundAnno);
        }

      }

      return std::move(result);
    };
  }
  else
  {
    return [&db, validAnnoKeys, constAnno, outputFilter](nodeid_t rhsNode) -> std::list<Annotation>
    {
      std::list<Annotation> result;
      // check all annotation keys
      for(AnnotationKey key : validAnnoKeys)
      {
       auto found = db.nodeAnnos.getAnnotations(rhsNode, key.ns, key.name);
       if(found && outputFilter({rhsNode, *found}))
       {
         if(constAnno)
         {
           result.push_back(*constAnno);
         }
         else
         {
          result.push_back(*found);
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

  if(node->type == ExecutionNodeType::nested_loop || (includeSeed && node->type == ExecutionNodeType::index_join))
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

std::uint64_t Plan::calculateNestedLoopProcessed(std::uint64_t outputLHS, std::uint64_t outputRHS)
{
  std::uint64_t processedInStep;
  if(outputLHS <= outputRHS)
  {
    // we use LHS as outer
    processedInStep = outputLHS + (outputLHS * outputRHS);
  }
  else
  {
    // we use RHS as outer
    processedInStep = outputRHS + (outputRHS * outputLHS);
  }
  return processedInStep;
}

uint64_t Plan::calculateIndexJoinProcessed(long double operatorSelectivity, uint64_t outputLHS, uint64_t outputRHS)
{
  // A index join processes each LHS and for each LHS the number of reachable nodes given by the operator.
  // The selectivity of the operator itself an estimation how many nodes are filtered out by the cross product.
  // We can use this number (without the edge annotation selectivity) to re-construct the number of reachable nodes.

  // avgReachable = (sel * cross) / lhs
  //              = (sel * lhs * rhs) / lhs
  //              = sel * rhs
  // processedInStep = lhs + (avgReachable * lhs)
  //                 = lhs + (sel * rhs * lhs)

  return
      static_cast<std::uint64_t>(
        (long double) outputLHS
        + (operatorSelectivity * (long double) outputRHS * (long double) outputLHS)
      );
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
    Operator::EstimationType estType = node->op->estimationType();
    if(estType == Operator::EstimationType::SELECTIVITY)
    {
      result += "{sel: " + std::to_string(node->op->selectivity());
    }
    else if(estType == Operator::EstimationType::MIN)
    {
      result += "{min";
    }
    else if(estType == Operator::EstimationType::MAX)
    {
      result += "{max";
    }
    else
    {
      result += "{";
    }

    if((node->type == ExecutionNodeType::index_join || node->type == ExecutionNodeType::nested_loop)
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
    case ExecutionNodeType::index_join:
      return "index_join";
    default:
      return "<unknown>";
  }

}


Plan::~Plan()
{
}

