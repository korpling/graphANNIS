#ifndef POINTING_H
#define POINTING_H

#include "abstractedgeoperator.h"

namespace annis
{

class Pointing : public AbstractEdgeOperator
{
public:
  Pointing(const DB& db, std::string ns, std::string name,
                   unsigned int minDistance = 1, unsigned int maxDistance = 1);

  Pointing(const DB& db, std::string ns, std::string name,
                   const Annotation& edgeAnno = Init::initAnnotation());

  virtual ~Pointing();
private:

};
} // end namespace annis
#endif // POINTING_H
