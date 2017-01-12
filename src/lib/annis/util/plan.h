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

#include <stddef.h>                     // for size_t
#include <stdint.h>                     // for uint64_t
#include <boost/optional/optional.hpp>  // for optional
#include <functional>                   // for function
#include <list>                         // for list
#include <map>                          // for map
#include <memory>                       // for shared_ptr
#include <string>                       // for string
#include <utility>                      // for pair
#include <vector>                       // for vector
#include "annis/types.h"                // for Annotation, nodeid_t, Match (...

namespace annis { class AnnotationKeySearch; }
namespace annis { class AnnotationSearch; }
namespace annis { class DB; }
namespace annis { class EstimatedSearch; }
namespace annis { class Iterator; }
namespace annis { class Operator; }
namespace annis { struct QueryConfig; }

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
  : output(0), intermediateSum(0), processedInStep(0)
  {}
  
  ExecutionEstimate(std::uint64_t output, std::uint64_t intermediateSum, std::uint64_t processedInStep)
    : output(output), intermediateSum(intermediateSum), processedInStep(processedInStep)
  {}
  
  const std::uint64_t output;
  const std::uint64_t intermediateSum;
  const std::uint64_t processedInStep;
};

struct ExecutionNode
{
  ExecutionNodeType type;
  
  std::shared_ptr<Iterator> join;  
  std::shared_ptr<Operator> op;
  size_t operatorIdx;
  std::map<size_t, size_t> nodePos;
  size_t componentNr;
  /** Only valid for seed join types */
  size_t numOfBackgroundTasks;
  
  std::shared_ptr<ExecutionNode> lhs;
  std::shared_ptr<ExecutionNode> rhs;

  std::shared_ptr<ExecutionEstimate> estimate;
  
  std::string description;
};


class Plan
{
public:
  Plan(std::shared_ptr<ExecutionNode> root);
  
  Plan(const Plan& orig);
  virtual ~Plan();
  
  bool executeStep(std::vector<Match>& result);
  double getCost();

  std::map<size_t, size_t> getOptimizedParallelizationMapping(const DB &db, QueryConfig config);
  
  static std::shared_ptr<ExecutionNode> join(std::shared_ptr<Operator> op,
    size_t lhsNodeNr, size_t rhsNodeNr,
    std::shared_ptr<ExecutionNode>, std::shared_ptr<ExecutionNode> rhs,
    const DB& db,
    bool forceNestedLoop, size_t numOfBackgroundTasks,
    QueryConfig config);
  
  std::string debugString() const;
  
  static std::function<std::list<Annotation> (nodeid_t)> createSearchFilter(const DB& db,
    std::shared_ptr<EstimatedSearch> search);

  static bool searchFilterReturnsMaximalOneAnno(std::shared_ptr<EstimatedSearch> search);
  
private:
  std::shared_ptr<ExecutionNode> root;
  
private:
  static std::shared_ptr<ExecutionEstimate> estimateTupleSize(std::shared_ptr<ExecutionNode> node);
  static void clearCachedEstimate(std::shared_ptr<ExecutionNode> node);
  
  std::string debugStringForNode(std::shared_ptr<const ExecutionNode> node, std::string indention) const;
  std::string typeToString(ExecutionNodeType type) const;
  
  static std::list<std::shared_ptr<ExecutionNode>> getDescendentNestedLoops(std::shared_ptr<ExecutionNode> node);

  static std::function<std::list<Annotation> (nodeid_t)> createAnnotationSearchFilter(
      const DB& db, std::shared_ptr<AnnotationSearch> annoSearch,
      boost::optional<Annotation> constAnno = boost::optional<Annotation>());

  static std::function<std::list<Annotation> (nodeid_t)> createAnnotationKeySearchFilter(
      const DB& db, std::shared_ptr<AnnotationKeySearch> annoKeySearch,
      boost::optional<Annotation> constAnno = boost::optional<Annotation>());

  static std::pair<std::shared_ptr<ExecutionNode>, uint64_t> findLargestProcessedInStep(
      std::shared_ptr<ExecutionNode> node, bool includeSeed = true);
};

} // end namespace annis
