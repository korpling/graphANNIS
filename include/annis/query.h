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
  Query(const DB& db);
  
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

  /**
   * Do some query optimizations based on simple heuristics.
   */
  void optimize();
  
  bool hasNext();
  std::vector<Match> next();
  
  virtual ~Query();

private:

  const DB& db;

  std::shared_ptr<Plan> bestPlan;
  std::vector<std::shared_ptr<AnnoIt>> nodes;
  std::list<OperatorEntry> operators;

  std::set<AnnotationKey> emptyAnnoKeySet;

private:
  void internalInit();
  
  static std::shared_ptr<Plan> createPlan(const std::vector<std::shared_ptr<AnnoIt>>& nodes, const std::list<OperatorEntry>& operators, const DB& db);

  static void addJoin(std::vector<std::shared_ptr<AnnoIt>>& source, const DB& db, const OperatorEntry &e, bool filterOnly = false);

  static void mergeComponents(std::map<int, int>& querynode2component, int c1, int c2);
  
};

} // end namespace annis
