#ifndef COMPAREFUNCTIONS_H
#define COMPAREFUNCTIONS_H

#include "edgedb.h"
#include "types.h"
#include <string.h>

#include <tuple>

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
    return std::tie(a.name, a.ns, a.val) < std::tie(b.name, b.ns, b.val);
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
    return std::tie(a.source, a.target) < std::tie(b.source, b.target);
  }
};

struct compTextProperty
{
  bool operator()(const struct TextProperty &a, const struct TextProperty &b) const
  {
    return std::tie(a.textID, a.val) < std::tie(b.textID, b.val);
  }
};

struct compRelativePosition
{
  bool operator()(const struct RelativePosition &a, const struct RelativePosition &b) const
  {
    return std::tie(a.root, a.pos) < std::tie(b.root, b.pos);
  }
};

struct compMatch
{
  bool operator()(const struct Match &a, const struct Match &b) const
  {
    return std::tie(a.node, a.anno.name, a.anno.ns, a.anno.val) < std::tie(b.node, b.anno.name, b.anno.ns, b.anno.val);
  }
};

struct compBinaryMatch
{
  bool operator()(const struct BinaryMatch &a, const struct BinaryMatch &b) const
  {
    return std::tie(a.left.node, a.left.anno.name, a.left.anno.ns, a.left.anno.val, a.right.node, a.right.anno.name, a.right.anno.ns, a.right.anno.val)
        <
        std::tie(b.left.node, b.left.anno.name, b.left.anno.ns, b.left.anno.val, b.right.node, b.right.anno.name, b.right.anno.ns, b.right.anno.val);
  }
};



} // end namespace annis

#endif // COMPAREFUNCTIONS_H
