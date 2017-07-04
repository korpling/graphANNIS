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

#include "singlealternativequery.h"
#include <annis/annosearch/estimatedsearch.h>      // for EstimatedSearch
#include <annis/annosearch/nodebyedgeannosearch.h>  // for NodeByEdgeAnnoSearch
#include <annis/annosearch/exactannokeysearch.h>
#include <annis/annosearch/regexannosearch.h>
#include <annis/db.h>                               // for DB
#include <annis/iterators.h>                        // for AnnoIt
#include <annis/operators/abstractedgeoperator.h>   // for AbstractEdgeOperator
#include <annis/operators/operator.h>               // for Operator
#include <annis/wrapper.h>                          // for ConstAnnoWrapper
#include <stdint.h>                                 // for int64_t
#include <algorithm>                                // for next_permutation
#include <boost/optional/optional.hpp>              // for optional
#include <iostream>                                 // for operator<<, basic...
#include <list>                                     // for list
#include <random>                                   // for mt19937, uniform_...
#include <utility>                                  // for pair
#include <vector>                                   // for vector
#include "annis/annostorage.h"                      // for AnnoStorage
#include "annis/queryconfig.h"                      // for QueryConfig
#include "annis/types.h"                            // for nodeid_t, Match
#include <annis/util/plan.h>                        // for Plan, ExecutionNode

using namespace annis;

SingleAlternativeQuery::SingleAlternativeQuery(const DB &db, QueryConfig config)
  : db(db), config(config)
{
}

SingleAlternativeQuery::~SingleAlternativeQuery() {
  
}

size_t SingleAlternativeQuery::addNode(std::shared_ptr<EstimatedSearch> n, bool wrapAnyNodeAnno)
{
  bestPlan.reset();

  size_t idx = nodes.size();

  if(wrapAnyNodeAnno)
  {
    Annotation constAnno = {db.getNodeTypeStringID(), db.getNamespaceStringID(), 0};
    n->setConstAnnoValue(constAnno);
  }

  nodes.push_back(n);

  return idx;
}

void SingleAlternativeQuery::addFilter(size_t node, std::function<bool (const Match &)> filterFunc, std::string description)
{
  filtersByNode.insert({node, {filterFunc, description}});
}

void SingleAlternativeQuery::addOperator(std::shared_ptr<Operator> op, size_t idxLeft, size_t idxRight, bool forceNestedLoop)
{
  bestPlan.reset();

  OperatorEntry entry;
  entry.op = op;
  entry.forceNestedLoop = forceNestedLoop;
  entry.idxLeft = idxLeft;
  entry.idxRight = idxRight;
  entry.originalOrder = operators.size();
  
  operators.push_back(entry);
}

void SingleAlternativeQuery::optimizeOperandOrder()
{
  if(!bestPlan && db.nodeAnnos.hasStatistics())
  {
    // for each commutative operator check if is better to switch the operands
    for(auto& e : operators)
    {
      if(e.op && e.op->isCommutative() && e.idxLeft < nodes.size() && e.idxRight < nodes.size())
      {
        std::shared_ptr<EstimatedSearch> lhs = 
          std::dynamic_pointer_cast<EstimatedSearch>(nodes[e.idxLeft]);
        std::shared_ptr<EstimatedSearch> rhs = 
          std::dynamic_pointer_cast<EstimatedSearch>(nodes[e.idxRight]);
        
        if(lhs && rhs)
        {
          std::int64_t estimateLHS = lhs->guessMaxCount();
          std::int64_t estimateRHS = rhs->guessMaxCount();
          
          if(estimateLHS >= 0 && estimateRHS >= 0 && estimateLHS > estimateRHS)
          {
            // the left one is larger, so switch both operands
            size_t oldLeft = e.idxLeft;
            e.idxLeft = e.idxRight;
            e.idxRight = oldLeft;
          }

        }
      }
    }
  }
}

void SingleAlternativeQuery::optimizeEdgeAnnoUsage()
{
  for(const OperatorEntry& opEntry : operators)
  {
    if(opEntry.idxLeft < nodes.size())
    {
      std::shared_ptr<EstimatedSearch> lhsNodeIt = std::dynamic_pointer_cast<EstimatedSearch>(nodes[opEntry.idxLeft]);
      std::shared_ptr<AbstractEdgeOperator> op = std::dynamic_pointer_cast<AbstractEdgeOperator>(opEntry.op);
      if(op && lhsNodeIt
         && !std::dynamic_pointer_cast<NodeByEdgeAnnoSearch>(lhsNodeIt))
      {
        std::int64_t guessedCountEdgeAnno = op->guessMaxCountEdgeAnnos();
        std::int64_t guessedCountNodeAnno = lhsNodeIt->guessMaxCount();
        if(guessedCountEdgeAnno >= 0 && guessedCountNodeAnno >= 0)
        {
          if(guessedCountEdgeAnno < guessedCountNodeAnno)
          {
            // it is more efficient to fetch the base node by searching for the edge annotation
            nodes[opEntry.idxLeft] = op->createAnnoSearch(Plan::createSearchFilter(db, lhsNodeIt),
                                                          Plan::searchFilterReturnsMaximalOneAnno(lhsNodeIt),
                                                          guessedCountNodeAnno,
                                                          lhsNodeIt->debugString());
          }
        }
      }
    }
  }
}

std::shared_ptr<const Plan> SingleAlternativeQuery::getBestPlan()
{
  if(!bestPlan)
  {
    internalInit();
  }
  return bestPlan;
}


std::shared_ptr<Plan> SingleAlternativeQuery::createPlan(const std::vector<std::shared_ptr<AnnoIt> >& nodes,
                                                         const std::vector<OperatorEntry>& operators,
                                                         std::map<size_t, std::shared_ptr<ExecutionEstimate>>& baseEstimateCache,
                                                         std::map<size_t, size_t> parallelizationMapping)
{
  std::map<nodeid_t, size_t> node2component;
  std::map<size_t, std::shared_ptr<ExecutionNode>> component2exec;

  // 1. add all nodes
  size_t i=0;
  for(auto& n : nodes)
  {
    std::shared_ptr<ExecutionNode> baseNode = std::make_shared<ExecutionNode>();
    baseNode->type = ExecutionNodeType::base;
    baseNode->nodePos[i] = 0;
    baseNode->componentNr = i;
    baseNode->join = n;

    auto itBaseEstimate = baseEstimateCache.find(i);
    if(itBaseEstimate == baseEstimateCache.end())
    {
      // calculate the estimation for the base node
      baseEstimateCache[i] = Plan::estimateTupleSize(baseNode);
    }
    else
    {
      // re-use already existing estimation
      baseNode->estimate = itBaseEstimate->second;
    }

    node2component[i] = i;
    component2exec[i] = baseNode;

    // add additional filters
    auto itFilterRange = filtersByNode.equal_range(i);
    std::list<std::function<bool(const Match &)>> filterList;
    for(auto it=itFilterRange.first; it != itFilterRange.second; it++)
    {
      filterList.push_back(it->second.first);
      // TODO: add description
    }
    if(!filterList.empty())
    {
      n->setOutputFilter(filterList);
    }

    i++;
  }
  const size_t numOfNodes = i;

  // 2. add the operators which produce the results
  for(size_t operatorIdx=0; operatorIdx < operators.size(); operatorIdx++)
  {
    auto& e = operators[operatorIdx];
    if(e.idxLeft < numOfNodes && e.idxRight < numOfNodes)
    {
      
      size_t componentLeft = node2component[e.idxLeft];
      size_t componentRight = node2component[e.idxRight];
      
      std::shared_ptr<ExecutionNode> execLeft = component2exec[componentLeft];
      std::shared_ptr<ExecutionNode> execRight = component2exec[componentRight];

      size_t numOfBackgroundTasks = 0;
      auto itParallelMapping = parallelizationMapping.find(operatorIdx);
      if(itParallelMapping != parallelizationMapping.end())
      {
        numOfBackgroundTasks = itParallelMapping->second;
      }

      std::shared_ptr<ExecutionNode> joinExec = Plan::join(e.op, e.idxLeft, e.idxRight,
          execLeft, execRight, db, e.forceNestedLoop, numOfBackgroundTasks, config);

      joinExec->operatorIdx = operatorIdx;

      updateComponentForNodes(node2component, componentLeft, joinExec->componentNr);
      updateComponentForNodes(node2component, componentRight, joinExec->componentNr);
      component2exec[joinExec->componentNr] = joinExec;      
      
    }
  }
  
   // 3. check if there is only one component left (all nodes are connected)
  boost::optional<size_t> firstComponentID;
  for(const auto& e : node2component)
  {
    if(!firstComponentID)
    {
      firstComponentID = e.second;
    }
    else
    {
      if(firstComponentID && *firstComponentID != e.second)
      {
        std::cerr << "Nodes  are not completly connected, failing" << std::endl;
        return std::shared_ptr<Plan>();
      }
    }
  }
  
  return std::make_shared<Plan>(component2exec[*firstComponentID]);
}

void SingleAlternativeQuery::optimizeUnboundRegex()
{
  if(!bestPlan)
  {
    for(size_t i=0; i < nodes.size(); i++)
    {
      std::shared_ptr<AnnoIt> n = nodes[i];
      std::shared_ptr<RegexAnnoSearch> regexSearch = std::dynamic_pointer_cast<RegexAnnoSearch>(n);

      // for each regex search test if the value is unbound
      if(regexSearch != nullptr && regexSearch->valueMatchesAllStrings())
      {
        // replace the regex search with an anno key search
        std::shared_ptr<ExactAnnoKeySearch> annoKeySearch;
        auto ns = regexSearch->getAnnoKeyNamespace();
        auto name = regexSearch->getAnnoKeyName();
        if(ns)
        {
          annoKeySearch = std::make_shared<ExactAnnoKeySearch>(db, *ns, name);
        }
        else
        {
          annoKeySearch = std::make_shared<ExactAnnoKeySearch>(db, name);
        }

        nodes[i] = annoKeySearch;
      }
    }
  }
}

void SingleAlternativeQuery::updateComponentForNodes(std::map<nodeid_t, size_t>& node2component, size_t from, size_t to)
{
  if(from == to)
  {
    // nothing todo
    return;
  }

  std::vector<int> nodeIDs2update;
  for(const auto e : node2component)
  {
    if(e.second == from)
    {
      nodeIDs2update.push_back(e.first);
    }
  }
  // set the component id for each node of the other component
  for(auto nodeID : nodeIDs2update)
  {
    node2component[nodeID] = to;
  }
}



void SingleAlternativeQuery::internalInit()
{
  if(bestPlan) {
    return;
  }
  std::map<size_t, std::shared_ptr<ExecutionEstimate>> baseEstimateCache;

  if(config.optimize)
  {

    optimizeUnboundRegex();

    ///////////////////////////////////////////////////////////
    // make sure all smaller operand are on the left side //
    ///////////////////////////////////////////////////////////
    optimizeOperandOrder();

    optimizeEdgeAnnoUsage();
    
    if(operators.size() > 1)
    {
      ////////////////////////////////////
      // 2. optimize the order of joins //
      ////////////////////////////////////
      if(operators.size() <= 6)
      {
        optimizeJoinOrderAllPermutations(baseEstimateCache);
      }
      else
      {
        optimizeJoinOrderRandom(baseEstimateCache);
      }
      
    } // end optimize join order
    else
    {
      bestPlan = createPlan(nodes, operators, baseEstimateCache);
      // still get the cost so the estimates are calculated
      bestPlan->getCost();
    }

    if(config.numOfBackgroundTasks >= 2)
    {
      std::map<size_t, size_t> parallelizationMapping = bestPlan->getOptimizedParallelizationMapping(db, config);
      // recreate the plan with the mapping
      bestPlan = createPlan(nodes, operators, baseEstimateCache, parallelizationMapping);
      // still get the cost so the estimates are calculated
      bestPlan->getCost();
    }
  }
  else
  {
    // create unoptimized plan
    bestPlan = createPlan(nodes, operators, baseEstimateCache);
  }
  
  currentResult.resize(nodes.size());
}

void SingleAlternativeQuery::optimizeJoinOrderRandom(std::map<size_t, std::shared_ptr<ExecutionEstimate>>& baseEstimateCache)
{
  // use a constant seed to make the result deterministic
  std::mt19937 randGen(4711);
  std::uniform_int_distribution<> dist(0, static_cast<int>(operators.size()-1));
    
  std::vector<OperatorEntry> optimizedOperators = operators;
  bestPlan = createPlan(nodes, optimizedOperators, baseEstimateCache);
  double bestCost = bestPlan->getCost();

//  std::cout << "orig plan:" << std::endl;
//  std::cout << operatorOrderDebugString(optimizedOperators) << std::endl;
//  std::cout << bestPlan->debugString() << std::endl;
//  std::cout << "-----------------------" << std::endl;

  // repeat until best plan is found
  const size_t numNewGenerations = 4;
  const size_t maxUnsuccessfulTries = 5*operators.size();
  size_t unsuccessful = 0;
  do
  {
    std::vector<std::vector<OperatorEntry>> familyOperators;
    familyOperators.reserve(numNewGenerations+1);

    familyOperators.push_back(optimizedOperators);

    for(size_t i = 0; i < numNewGenerations; i++)
    {
      // use the the previous generation as basis
      std::vector<OperatorEntry> tmpOperators = familyOperators[i];
      // randomly select two joins,
      int a, b;
      do
      {
        a = dist(randGen);
        b = dist(randGen);
      } while(a == b);

      // switch the order of the selected joins
      std::swap(tmpOperators[a], tmpOperators[b]);
      familyOperators.push_back(tmpOperators);
    }

    bool foundBetterPlan = false;
    for(size_t i = 1; i < familyOperators.size(); i++)
    {
      auto altPlan = createPlan(nodes, familyOperators[i], baseEstimateCache);
      double altCost = altPlan->getCost();

//      std::cout << "................................" << std::endl;
//      std::cout << "testing new operator order" << std::endl;
//      std::cout << operatorOrderDebugString(familyOperators[i]) << std::endl;
//      std::cout << altPlan->debugString() << std::endl;
//      std::cout << "................................" << std::endl;

      if(altCost < bestCost)
      {
        bestPlan = altPlan;
        optimizedOperators = familyOperators[i];

        foundBetterPlan = true;
//        std::cout << "================================" << std::endl;
//        std::cout << "new plan:" << std::endl;
//        std::cout << operatorOrderDebugString(optimizedOperators) << std::endl;
//        std::cout << bestPlan->debugString() << std::endl;
//        std::cout << "================================" << std::endl;

        bestCost = altCost;
        unsuccessful = 0;
      }
    }

    if(!foundBetterPlan)
    {
      unsuccessful++;
    }

  } while(unsuccessful < maxUnsuccessfulTries);

  operators = optimizedOperators;
}

void SingleAlternativeQuery::optimizeJoinOrderAllPermutations(std::map<size_t, std::shared_ptr<ExecutionEstimate>>& baseEstimateCache)
{
  // make sure the first permutation is the sorted one
  std::vector<OperatorEntry> testOrder = operators;
  std::sort(testOrder.begin(), testOrder.end(), compare_opentry_origorder);
  
  bestPlan = createPlan(nodes, testOrder, baseEstimateCache);
  operators = testOrder;

//  bestPlan->getCost();
//  std::cout << operatorOrderDebugString(testOrder) << std::endl;
//  std::cout << bestPlan->debugString() << std::endl;
//  std::cout << "-------------------------------" << std::endl;
  
  while(std::next_permutation(testOrder.begin(), testOrder.end(), compare_opentry_origorder))
  {
    std::shared_ptr<Plan> testPlan = createPlan(nodes, testOrder, baseEstimateCache);
//    testPlan->getCost();
//    std::cout << operatorOrderDebugString(testOrder) << std::endl;
//    std::cout << testPlan->debugString() << std::endl;
    
    if(testPlan->getCost() < bestPlan->getCost())
    {
      bestPlan = testPlan;
      operators = testOrder;
      
//      std::cout << "!!!new best join order!!! " << std::endl;
    }
//    std::cout << "-------------------------------" << std::endl;
  }
}


std::string SingleAlternativeQuery::operatorOrderDebugString(const std::vector<OperatorEntry>& ops)
{
  std::string result = "";
  for(auto it=ops.begin(); it != ops.end(); it++)
  {
    if(it != ops.begin())
    {
      result += " | ";
    }
    if(it->op)
    {
      result += "#" + std::to_string(it->idxLeft+1) + " " +
        it->op->description()
        + " #" + std::to_string(it->idxRight+1);
    }
    else
    {
      result += "<empty>";
    }
  }
  
  return result;
}



bool SingleAlternativeQuery::next()
{
  if(!bestPlan)
  {
    internalInit();
  }
  
  if(bestPlan)
  {
    return bestPlan->executeStep(currentResult);
  }
  else
  {
    return false;
  }
}


