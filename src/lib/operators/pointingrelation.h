#ifndef POINTINGRELATION_H
#define POINTINGRELATION_H

#include "abstractedgeoperator.h"

namespace annis
{

class PointingRelation : public AbstractEdgeOperator
{
public:
  PointingRelation(const DB& db, std::string ns, std::string name,
                   unsigned int minDistance = 1, unsigned int maxDistance = 1);

  PointingRelation(const DB& db, std::string ns, std::string name,
                   const Annotation& edgeAnno = Init::initAnnotation());

  virtual ~PointingRelation();
private:

};
} // end namespace annis
#endif // POINTINGRELATION_H
