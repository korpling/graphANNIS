#include <annis/operators/pointing.h>

using namespace annis;

Pointing::Pointing(GraphStorageHolder& gsh, const StringStorage& strings, std::string ns, std::string name,
                                   unsigned int minDistance, unsigned int maxDistance)
  : AbstractEdgeOperator(ComponentType::POINTING,
                         gsh, strings, ns, name, minDistance, maxDistance)
{
}

Pointing::Pointing(GraphStorageHolder &gsh, const StringStorage &strings, std::string ns, std::string name, const Annotation &edgeAnno)
  : AbstractEdgeOperator(ComponentType::POINTING,
                         gsh, strings, ns, name, edgeAnno)
{
}

Pointing::~Pointing()
{

}



