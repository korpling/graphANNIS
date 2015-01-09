#include "pointingrelation.h"
#include "wrapper.h"

using namespace annis;

PointingRelation::PointingRelation(const DB &db, std::string ns, std::string name,
                                   unsigned int minDistance, unsigned int maxDistance)
  : AbstractEdgeOperator(ComponentType::POINTING,
                         db, ns, name, minDistance, maxDistance)
{
}

PointingRelation::PointingRelation(const DB &db, std::string ns, std::string name, const Annotation &edgeAnno)
  : AbstractEdgeOperator(ComponentType::POINTING,
                         db, ns, name, edgeAnno)
{
}

PointingRelation::~PointingRelation()
{

}



