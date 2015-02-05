#ifndef ABSTRACTEDGEOPERATOR_H
#define ABSTRACTEDGEOPERATOR_H

#include "../db.h"
#include "../graphstorage.h"
#include "../operator.h"
#include <vector>

namespace annis
{

class AbstractEdgeOperator : public Operator
{
public:
  AbstractEdgeOperator(
      ComponentType componentType,
      const DB& db, std::string ns, std::string name,
      unsigned int minDistance = 1, unsigned int maxDistance = 1);

  AbstractEdgeOperator(
      ComponentType componentType,
      const DB& db, std::string ns, std::string name,
      const Annotation& edgeAnno = Init::initAnnotation());

  virtual std::unique_ptr<AnnoIt> retrieveMatches(const Match& lhs);
  virtual bool filter(const Match& lhs, const Match& rhs);

  virtual ~AbstractEdgeOperator();
private:
  ComponentType componentType;
  const DB& db;
  std::string ns;
  std::string name;
  unsigned int minDistance;
  unsigned int maxDistance;
  Annotation anyAnno;
  const Annotation edgeAnno;

  std::vector<const ReadableGraphStorage*> edb;

  void initEdgeDB();
  bool checkEdgeAnnotation(const ReadableGraphStorage *e, nodeid_t source, nodeid_t target);
};

} // end namespace annis
#endif // ABSTRACTEDGEOPERATOR_H
