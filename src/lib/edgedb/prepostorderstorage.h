#ifndef PREPOSTORDERSTORAGE_H
#define PREPOSTORDERSTORAGE_H

#include "fallbackedgedb.h"

namespace annis
{

class PrePostOrderStorage
{
public:
  PrePostOrderStorage();
  virtual ~PrePostOrderStorage();

  virtual void calculateIndex();

private:


};

} // end namespace annis

#endif // PREPOSTORDERSTORAGE_H
