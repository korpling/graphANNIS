#include <annis/operators/pointing.h>

using namespace annis;

Pointing::Pointing(const DB &db, std::string ns, std::string name,
                                   unsigned int minDistance, unsigned int maxDistance)
  : AbstractEdgeOperator(ComponentType::POINTING,
                         db, ns, name, minDistance, maxDistance)
{
}

Pointing::Pointing(const DB &db, std::string ns, std::string name, const Annotation &edgeAnno)
  : AbstractEdgeOperator(ComponentType::POINTING,
                         db, ns, name, edgeAnno)
{
}

Pointing::~Pointing()
{

}



