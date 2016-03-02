#pragma once

#include <annis/types.h>

namespace annis
{

class AnnoIt
{
public:
  virtual bool next(Match& m) = 0;
  virtual void reset() = 0;

  virtual ~AnnoIt() {}
};

class EdgeIterator
{
public:
  virtual std::pair<bool, nodeid_t> next() = 0;
  virtual void reset() = 0;

  virtual ~EdgeIterator() {}
};

class BinaryIt
{
public:
  virtual bool next(Match& lhsMatch, Match& rhsMatch) = 0;
  virtual void reset() = 0;

  virtual ~BinaryIt() {}
};

} // end namespace annis
