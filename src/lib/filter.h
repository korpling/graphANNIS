#ifndef FILTER_H
#define FILTER_H

#include <iterators.h>
#include <operators/operator.h>

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

#endif // FILTER_H
