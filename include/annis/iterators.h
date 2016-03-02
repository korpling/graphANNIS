#pragma once

#include <annis/types.h>

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
  virtual bool next(Match& lhsMatch, Match& rhsMatch) = 0;
  virtual void reset() = 0;

  virtual ~Iterator() {}
};

class AnnoIt : public Iterator
{
public:
  virtual bool next(Match& m) = 0;
  
  virtual bool next(Match& lhsMatch, Match& rhsMatch) override
  {
    bool found = next(lhsMatch);
    if(found)
    {
      rhsMatch = lhsMatch;
    }
    return found;
  }

  virtual ~AnnoIt() {}
};

} // end namespace annis
