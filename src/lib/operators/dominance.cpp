#include <annis/operators/dominance.h>

using namespace annis;

Dominance::Dominance(GraphStorageHolder& gsh, const StringStorage& strings, std::string ns, std::string name,
                                   unsigned int minDistance, unsigned int maxDistance)
  : AbstractEdgeOperator(ComponentType::DOMINANCE,
                         gsh, strings, ns, name, minDistance, maxDistance)
{
}

Dominance::Dominance(GraphStorageHolder& gsh, const StringStorage& strings, std::string ns, std::string name, const Annotation &edgeAnno)
  : AbstractEdgeOperator(ComponentType::DOMINANCE,
                         gsh, strings, ns, name, edgeAnno)
{
}

Dominance::~Dominance()
{

}

