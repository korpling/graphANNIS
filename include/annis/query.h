#pragma once

#include <memory>
#include <vector>
#include <list>
#include <set>
#include <map>

#include <annis/types.h>
#include <annis/util/plan.h>

namespace annis
{
  
class Operator;
class DB;
class AnnoIt;
class AnnotationSearch;
class AnnotationKeySearch;

struct OperatorEntry
{
  std::shared_ptr<Operator> op;
  size_t idxLeft;
  size_t idxRight;
  bool useNestedLoop;
};

class Query
{
public:
  Query(const DB& db, bool optimize = true);
  
  /**
   * @brief Add a new node to query
   * @param n The initial source
   * @return new node number
   */
  size_t addNode(std::shared_ptr<AnnotationSearch> n, bool wrapAnyNodeAnno = false);
  size_t addNode(std::shared_ptr<AnnotationKeySearch> n, bool wrapAnyNodeAnno = false);

  /**
   * @brief add an operator to the execution queue
   * @param op
   * @param idxLeft index of LHS node
   * @param idxRight index of RHS node
   * @param useNestedLoop if true a nested loop join is used instead of the default "seed join"
   */
  void addOperator(std::shared_ptr<Operator> op, size_t idxLeft, size_t idxRight, bool useNestedLoop = false);
  
  bool next();
  
  const std::vector<Match>& getCurrent() { return currentResult;}
  
  virtual ~Query();

private:

  const DB& db;
  bool optimize;
  
  std::vector<Match> currentResult;

  std::shared_ptr<Plan> bestPlan;
  std::vector<std::shared_ptr<AnnoIt>> nodes;
  std::vector<OperatorEntry> operators;

  std::set<AnnotationKey> emptyAnnoKeySet;

private:
  void internalInit();
  
  std::shared_ptr<Plan> createPlan(const std::vector<std::shared_ptr<AnnoIt>>& nodes, const std::vector<OperatorEntry>& operators);
  
  void optimizeOperandOrder();
  
};

} // end namespace annis
