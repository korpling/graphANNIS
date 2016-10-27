#pragma once

#include <annis/iterators.h>
#include <annis/operators/operator.h>

namespace annis
{

class Filter : public Iterator
{
public:

  Filter(std::shared_ptr<Operator> op, std::shared_ptr<Iterator> inner,
    size_t lhsIdx, size_t rhsIdx);

  virtual bool next(std::vector<Match>& tuple) override;
  virtual void reset() override;

  virtual ~Filter();

private:
  std::shared_ptr<Operator> op;
  std::shared_ptr<Iterator> inner;
  size_t lhsIdx; 
  size_t rhsIdx;
};

} // end namespace annis
