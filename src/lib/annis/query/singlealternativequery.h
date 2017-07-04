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

#pragma once

#include <annis/queryconfig.h>  // for QueryConfig
#include <annis/types.h>        // for AnnotationKey, Match, nodeid_t

#include <stddef.h>             // for size_t
#include <map>                  // for map
#include <memory>               // for shared_ptr
#include <set>                  // for set
#include <string>               // for string
#include <vector>               // for vector
#include <functional>

namespace annis { class AnnoIt; }
namespace annis { class EstimatedSearch; }
namespace annis { class DB; }
namespace annis { class Operator; }
namespace annis { class Plan; }
namespace annis { class ExecutionEstimate; }

namespace annis
{

struct OperatorEntry
{
  std::shared_ptr<Operator> op;
  size_t idxLeft;
  size_t idxRight;
  bool forceNestedLoop;
  
  size_t originalOrder;
};

class SingleAlternativeQuery
{
public:
  SingleAlternativeQuery(const DB& db, QueryConfig config = QueryConfig());
  
  /**
   * @brief Add a new node to query
   * @param n The initial source
   * @return new node number
   */
  size_t addNode(std::shared_ptr<EstimatedSearch> n, bool wrapAnyNodeAnno = false);

  void addFilter(size_t node, std::function<bool(const Match &)> filterFunc, std::string description="");

  /**
   * @brief add an operator to the execution queue
   * @param op
   * @param idxLeft index of LHS node
   * @param idxRight index of RHS node
   * @param forceNestedLoop if true a nested loop join is used instead of the default "seed join"
   */
  void addOperator(std::shared_ptr<Operator> op, size_t idxLeft, size_t idxRight, bool forceNestedLoop = false);
  
  bool next();
  const std::vector<Match>& getCurrent() { return currentResult;}
  
  std::shared_ptr<const Plan> getBestPlan();
  
  virtual ~SingleAlternativeQuery();

private:

  const DB& db;
  const QueryConfig config;
  
  std::vector<Match> currentResult;

  std::shared_ptr<Plan> bestPlan;
  std::vector<std::shared_ptr<AnnoIt>> nodes;
  std::multimap<size_t, std::pair<std::function<bool(const Match &)>, std::string >> filtersByNode;
  std::vector<OperatorEntry> operators;

  std::set<AnnotationKey> emptyAnnoKeySet;

  struct CompareOperatorEntryOrigOrder
  {

    bool operator()(const OperatorEntry& o1, const OperatorEntry& o2)
    {
      return (o1.originalOrder < o2.originalOrder);
    }
  } compare_opentry_origorder;

private:
  void internalInit();
  
  std::shared_ptr<Plan> createPlan(const std::vector<std::shared_ptr<AnnoIt>>& nodes,
                                   const std::vector<OperatorEntry>& operators,
                                   std::map<size_t, std::shared_ptr<ExecutionEstimate>>& baseEstimateCache,
                                   std::map<size_t, size_t> parallelizationMapping = std::map<size_t,size_t>());
  
  void optimizeUnboundRegex();

  void optimizeOperandOrder();

  void optimizeEdgeAnnoUsage();
  
  void optimizeJoinOrderRandom(std::map<size_t, std::shared_ptr<ExecutionEstimate> > &baseEstimateCache);
  void optimizeJoinOrderAllPermutations(std::map<size_t, std::shared_ptr<ExecutionEstimate> > &baseEstimateCache);

  void updateComponentForNodes(std::map<nodeid_t, size_t>& node2component, size_t from, size_t to);
  
  std::string operatorOrderDebugString(const std::vector<OperatorEntry>& ops);
  
};

} // end namespace annis
