#pragma once

#include <annis/types.h>
#include <vector>
#include <list>
#include <functional>
#include <boost/optional.hpp>

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
  using MatchFilter = std::function<bool(const Match &)>;

  virtual bool next(Match& m) = 0;
  
  virtual bool next(std::vector<Match>& tuple) override
  {
    tuple.resize(1);
    if(outputFilter)
    {
      while(next(tuple[0]))
      {
        if((*outputFilter)(tuple[0]))
        {
          return true;
        }
      }
      return false;
    }
    else
    {
      return next(tuple[0]);
    }
  }

  void setOutputFilter(std::list<MatchFilter> filters)
  {
    if(filters.empty())
    {
      outputFilter.reset();
    }
    else
    {
      outputFilter = [filters] (const Match& m) -> bool
      {
        for(MatchFilter f : filters)
        {
          if(!f(m))
          {
            return false;
          }
        }
        return true;
      };
    }
  }

  MatchFilter getOutputFilter() const
  {
    if(outputFilter)
    {
      return *outputFilter;
    }
    else
    {
      return [] (const Match& /*m*/) -> bool {return true;};
    }
  }

  virtual ~AnnoIt() {}
private:
  boost::optional<MatchFilter> outputFilter;
};

} // end namespace annis
