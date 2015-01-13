#ifndef JOIN
#define JOIN

#include "iterators.h"
#include "iterators.h"

#include <memory>

namespace annis
{
class Join : public BinaryIt
{
public:
  virtual void init(std::shared_ptr<AnnoIt> lhs, std::shared_ptr<AnnoIt> rhs) = 0;

  virtual ~Join() {}
};
} // end namespacce

#endif // JOIN

