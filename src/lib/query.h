#ifndef QUERY_H
#define QUERY_H

#include <memory>
#include <vector>

#include <annotationiterator.h>

namespace annis
{

class Query
{
public:
  Query();

  /**
   * @brief Add a new node to query
   * @param n The initial source
   * @return new node number
   */
  size_t addNode(std::shared_ptr<AnnotationIterator> n);

  /**
   * @brief Execute an operator
   * @param op
   * @param idxLeft index of LHS node
   * @param idxRight index of RHS node
   */
  void executeOperator(std::shared_ptr<BinaryOperatorIterator> op, size_t idxLeft, size_t idxRight);

private:
  std::vector<std::shared_ptr<AnnotationIterator>> source;
};

} // end namespace annis
#endif // QUERY_H
