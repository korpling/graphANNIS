#pragma once

#include <annis/iterators.h>
#include <annis/operators/operator.h>

namespace annis
{

class Filter : public BinaryIt
{
public:

  Filter(std::shared_ptr<Operator> op, std::shared_ptr<AnnoIt> lhs, std::shared_ptr<AnnoIt> rhs);

  virtual BinaryMatch next();
  virtual void reset();

  virtual ~Filter();

private:
  std::shared_ptr<Operator> op;
  std::shared_ptr<AnnoIt> lhs;
  std::shared_ptr<AnnoIt> rhs;
};

} // end namespace annis
