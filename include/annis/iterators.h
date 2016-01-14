#pragma once

#include "types.h"

namespace annis
{

class AnnoIt
{
public:
  virtual bool hasNext() = 0;
  virtual Match next() = 0;
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
  virtual BinaryMatch next() = 0;
  virtual void reset() = 0;

  virtual ~BinaryIt() {}
};

} // end namespace annis
