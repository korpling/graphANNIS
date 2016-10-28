#pragma once

#include "abstractedgeoperator.h"

namespace annis
{

class Pointing : public AbstractEdgeOperator
{
public:
  Pointing(GraphStorageHolder &gsh, const StringStorage &strings, std::string ns, std::string name,
                   unsigned int minDistance = 1, unsigned int maxDistance = 1);

  Pointing(GraphStorageHolder& gsh, const StringStorage& strings, std::string ns, std::string name,
                   const Annotation& edgeAnno);

  virtual std::string operatorString() override
  {
    return "->";
  }
  
  virtual ~Pointing();
private:
};
} // end namespace annis
