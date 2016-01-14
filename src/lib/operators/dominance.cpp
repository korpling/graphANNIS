#include <annis/operators/dominance.h>

using namespace annis;

Dominance::Dominance(const DB &db, std::string ns, std::string name,
                                   unsigned int minDistance, unsigned int maxDistance)
  : AbstractEdgeOperator(ComponentType::DOMINANCE,
                         db, ns, name, minDistance, maxDistance)
{
}

Dominance::Dominance(const DB &db, std::string ns, std::string name, const Annotation &edgeAnno)
  : AbstractEdgeOperator(ComponentType::DOMINANCE,
                         db, ns, name, edgeAnno)
{
}

Dominance::~Dominance()
{

}

