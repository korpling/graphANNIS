#ifndef QUERY_H
#define QUERY_H

#include <memory>
#include <vector>
#include <list>

#include "db.h"
#include <annotationiterator.h>
#include "operator.h"
#include "operators/wrapper.h"

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
  Query(const DB& db);

  /**
   * @brief Add a new node to query
   * @param n The initial source
   * @return new node number
   */
  size_t addNode(std::shared_ptr<AnnoIt> n);

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

  const DB& db;

  std::vector<std::shared_ptr<AnnoIt>> source;
  std::list<std::shared_ptr<AnnoIt>> nodes;
  std::list<OperatorEntry> operators;

  bool initialized;

  void internalInit();

};

} // end namespace annis
#endif // QUERY_H
