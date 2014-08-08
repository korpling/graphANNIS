#ifndef COMPAREFUNCTIONS_H
#define COMPAREFUNCTIONS_H

#include "tupel.h"

namespace annis
{

struct compEdges
{
  bool operator()(const struct Edge &a, const struct Edge &b) const
  {
    // compare by source id
    if(a.source < b.source)
    {
      return true;
    }
    else if(a.source > b.source)
    {
      return false;
    }
    // if equal compare by target id
    if(a.target < b.target)
    {
      return true;
    }
    else if(a.target > b.target)
    {
      return false;
    }

    // if still equal compare by component id
    if(a.component < b.component)
    {
      return true;
    }
    else if(a.component > b.component)
    {
      return false;
    }

    // they are equal
    return true;
  }
};

} // end namespace annis

#endif // COMPAREFUNCTIONS_H
