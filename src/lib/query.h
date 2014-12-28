#ifndef QUERY_H
#define QUERY_H

#include <memory>
#include <vector>
#include <list>

#include <annotationiterator.h>
#include "operator.h"

namespace annis
{

struct OperatorEntry
{
  std::shared_ptr<Join> op;
  size_t idxLeft;
  size_t idxRight;
};

class Query
{
public:
  Query();

  /**
   * @brief Add a new node to query
   * @param n The initial source
   * @return new node number
   */
  size_t addNode(std::shared_ptr<CacheableAnnoIt> n);

  /**
   * @brief add an operator to the execution queue
   * @param op
   * @param idxLeft index of LHS node
   * @param idxRight index of RHS node
   */
  void addOperator(std::shared_ptr<Join> op, size_t idxLeft, size_t idxRight);

  /**
   * @brief add an operator to the execution queue
   * @param op
   * @param idxLeft index of LHS node
   * @param idxRight index of RHS node
   */
  void addOperator(std::shared_ptr<Operator> op, size_t idxLeft, size_t idxRight);

  bool hasNext();
  std::vector<Match> next();

private:
  std::vector<std::shared_ptr<CacheableAnnoIt>> source;
  /**
   * @brief Stores if a certain source is the original (and we should call "next()") or just a copy
   * where we have to use "current()"
   */
  std::vector<bool> isOrig;

  std::list<std::shared_ptr<CacheableAnnoIt>> nodes;
  std::list<OperatorEntry> operators;

  bool initialized;

  void internalInit();

};

} // end namespace annis
#endif // QUERY_H
