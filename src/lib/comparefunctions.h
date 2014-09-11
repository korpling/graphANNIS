#ifndef COMPAREFUNCTIONS_H
#define COMPAREFUNCTIONS_H

#include "edgedb.h"
#include "types.h"
#include <string.h>

namespace annis
{

struct compComponent
{
  bool operator()(const struct Component &a, const struct Component &b) const
  {
    // compare by type
    if(a.type < b.type)
    {
      return true;
    }
    else if(a.type > b.type)
    {
      return false;
    }
    // if equal compare by namespace
    int nsCompare = strncmp(a.ns, b.ns, MAX_COMPONENT_NAME_SIZE);
    if(nsCompare < 0)
    {
      return true;
    }
    else if(nsCompare > 0)
    {
      return false;
    }

    // if still equal compare by name
    int nameCompare = strncmp(a.name, b.name, MAX_COMPONENT_NAME_SIZE);
    if(nameCompare < 0)
    {
      return true;
    }
    else if(nameCompare > 0)
    {
      return false;
    }

    // they are equal
    return false;
  }
};

/*
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
    return false;
  }
};
*/

} // end namespace annis

#endif // COMPAREFUNCTIONS_H
