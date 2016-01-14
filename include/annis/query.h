#ifndef QUERY_H
#define QUERY_H

#include <memory>
#include <vector>
#include <list>
#include <set>

#include <annis/db.h>
#include <annis/iterators.h>
#include <annis/operators/operator.h>
#include <annis/wrapper.h>
#include <annis/annosearch/annotationsearch.h>

namespace annis
{

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

  bool hasNext();
  std::vector<Match> next();
  
  virtual ~Query();

private:

  const DB& db;

  std::vector<std::shared_ptr<AnnoIt>> source;
  std::vector<std::shared_ptr<AnnoIt>> nodes;
  std::list<OperatorEntry> operators;

  bool initialized;

  std::map<int, int> querynode2component;
  std::set<AnnotationKey> emptyAnnoKeySet;

private:
  void internalInit();

  void addJoin(OperatorEntry &e, bool filterOnly = false);

  void mergeComponents(int c1, int c2);
  

};

} // end namespace annis
#endif // QUERY_H
