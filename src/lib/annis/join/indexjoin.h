#pragma once

#include <annis/iterators.h>

namespace annis
{

class IndexJoin : public Iterator
{
public:
  IndexJoin();

  virtual bool next(std::vector<Match>& tuple);
  virtual void reset();

  virtual ~IndexJoin() {}
};
}

