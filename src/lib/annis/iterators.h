#pragma once

#include <annis/types.h>
#include <vector>

namespace annis
{

class EdgeIterator
{
public:
  virtual std::pair<bool, nodeid_t> next() = 0;
  virtual void reset() = 0;

  virtual ~EdgeIterator() {}
};

class Iterator
{
public:
  virtual bool next(std::vector<Match>& tuple) = 0;
  virtual void reset() = 0;

  virtual ~Iterator() {}
};

class AnnoIt : public Iterator
{
public:
  virtual bool next(Match& m) = 0;
  
  virtual bool next(std::vector<Match>& tuple) override
  {
    tuple.resize(1);
    return next(tuple[0]);
  }

  virtual ~AnnoIt() {}
};

} // end namespace annis
