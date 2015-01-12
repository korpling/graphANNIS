#ifndef DOMINANCE_H
#define DOMINANCE_H

#include "abstractedgeoperator.h"

namespace annis
{

class Dominance : public AbstractEdgeOperator
{
public:
  Dominance(const DB& db, std::string ns, std::string name,
                   unsigned int minDistance = 1, unsigned int maxDistance = 1);

  Dominance(const DB& db, std::string ns, std::string name,
                   const Annotation& edgeAnno);

  virtual ~Dominance();
private:

};
} // end namespace annis
#endif // DOMINANCE_H
