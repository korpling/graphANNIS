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
    int nsCompare = strncmp(a.layer, b.layer, MAX_COMPONENT_NAME_SIZE);
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

struct compAnno
{
  bool operator()(const struct Annotation &a, const struct Annotation &b) const
  {
    // compare by name (non lexical but just by the ID)
    if(a.name < b.name)
    {
      return true;
    }
    else if(a.name > b.name)
    {
      return false;
    }
    // if equal, compare by namespace (non lexical but just by the ID)
    if(a.ns < b.ns)
    {
      return true;
    }
    else if(a.ns > b.ns)
    {
      return false;
    }

    // if still equal compare by value (non lexical but just by the ID)
   if(a.val < b.val)
    {
      return true;
    }
    else if(a.val > b.val)
    {
      return false;
    }

    // they are equal
    return false;
  }
};

/**
 * @brief Compares two annotations.
 * A value of "0" in any fields of the annnnotation stands for "any value" and always compares to true.
 * @param a
 * @param b
 * @return True if the annotations are the same
 */
static bool checkAnnotationEqual(const struct Annotation &a, const struct Annotation &b)
{
  // compare by name (non lexical but just by the ID)
  if(a.name != 0 && b.name != 0 &&  a.name != b.name)
  {
    return false;
  }

  // if equal, compare by namespace (non lexical but just by the ID)
  if(a.ns != 0 & b.ns != 0 && a.ns != b.ns)
  {
    return false;
  }

  // if still equal compare by value (non lexical but just by the ID)
 if(a.val != 0 && b.val != 0 && a.val != b.val)
  {
    return false;
  }

  // they are equal
  return true;
}

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

    // they are equal
    return false;
  }
};

struct compTextProperty
{
  bool operator()(const struct TextProperty &a, const struct TextProperty &b) const
  {
    if(a.textID < b.textID)
    {
      return true;
    }
    else if(a.textID > b.textID)
    {
      return false;
    }
    if(a.val < b.val)
    {
      return true;
    }
    else if(a.val > b.val)
    {
      return false;
    }

    // they are equal
    return false;
  }
};

struct compRelativePosition
{
  bool operator()(const struct RelativePosition &a, const struct RelativePosition &b) const
  {
    if(a.root < b.root)
    {
      return true;
    }
    else if(a.root > b.root)
    {
      return false;
    }
    if(a.pos < b.pos)
    {
      return true;
    }
    else if(a.pos > b.pos)
    {
      return false;
    }

    // they are equal
    return false;
  }
};



} // end namespace annis

#endif // COMPAREFUNCTIONS_H
